//! Defines data & functions for displaying syntax types.

use super::*;

use std::collections::HashMap;
use std::convert::Infallible;

use decoration::AsUndecorated;
use expr::Visitable;

use pretty_decimal::PrettyDecimal;
use unicode_width::UnicodeWidthStr;

/// Context information to control the formatting of the transaction.
#[derive(Debug, Default, Clone)]
pub struct DisplayContext {
    default_commodity: CommodityDisplayOption,
    commodity_overrides: HashMap<String, CommodityDisplayOption>,
}

impl DisplayContext {
    /// Creates a new [`DisplayContext`].
    pub fn new(
        default_commodity: CommodityDisplayOption,
        commodity_overrides: HashMap<String, CommodityDisplayOption>,
    ) -> Self {
        Self {
            default_commodity,
            commodity_overrides,
        }
    }

    /// Returns given object reference wrapped with a context for `fmt::Display`.
    pub fn as_display<'a, T>(&'a self, value: &'a T) -> WithContext<'a, T>
    where
        WithContext<'a, T>: fmt::Display,
    {
        WithContext {
            value,
            context: self,
        }
    }

    /// Returns decimal format for the `commodity`.
    pub fn decimal_format(&self, commodity: &str) -> Option<pretty_decimal::Format> {
        self.commodity_overrides
            .get(commodity)
            .and_then(|o| o.format)
            .or(self.default_commodity.format)
    }

    /// Returns the minimum scale of the amount for the `commodity`.
    pub fn min_scale(&self, commodity: &str) -> Option<u8> {
        self.commodity_overrides
            .get(commodity)
            .and_then(|o| o.min_scale)
            .or(self.default_commodity.min_scale)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CommodityDisplayOption {
    pub format: Option<pretty_decimal::Format>,
    pub min_scale: Option<u8>,
}

/// Derives a [`DisplayContext`] out of the Ledger itself.
///
/// How an amount should be printed is a property of its commodity, not of the
/// particular literal, and the commodity may well be declared in another file
/// than the one being printed. So a formatter has to walk the whole tree first,
/// feeding every entry into this builder, and only then start printing.
///
/// Two sources are combined, and the declared one wins per field:
///
/// 1. `commodity ... format` directives, an explicit statement of intent.
/// 2. The amounts actually written in the files, for the commodities without
///    such a directive. Precision only ever grows: the widest literal wins,
///    and the "most formatted" style wins, so the result doesn't depend on
///    which file happened to be read first.
///
/// Costs (`@` / `@@`) and lot prices are deliberately *not* observed, as an
/// exchange rate routinely carries more digits than the commodity's own
/// precision, which would otherwise inflate every amount of that commodity.
#[derive(Debug, Default, Clone)]
pub struct DisplayContextBuilder {
    default_commodity: CommodityDisplayOption,
    commodities: HashMap<String, CommodityEstimate>,
}

/// What we know about one commodity, before deciding which source wins.
#[derive(Debug, Default, Clone, Copy)]
struct CommodityEstimate {
    /// Taken from a `commodity ... format` directive, or given by the caller.
    declared: CommodityDisplayOption,
    /// Inferred from the amount literals.
    observed: CommodityDisplayOption,
}

impl DisplayContextBuilder {
    /// Creates an empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the fallback used for the commodities the builder never saw.
    ///
    /// Note this also applies to the amounts without a commodity, so leave it
    /// alone unless the caller really means to touch those.
    pub fn with_default_commodity(self, default_commodity: CommodityDisplayOption) -> Self {
        Self {
            default_commodity,
            ..self
        }
    }

    /// Declares the `commodity` format explicitly, as a `commodity ... format`
    /// directive would do.
    pub fn declare<T>(&mut self, commodity: T, option: CommodityDisplayOption)
    where
        T: Into<String>,
    {
        let declared = &mut self
            .commodities
            .entry(commodity.into())
            .or_default()
            .declared;
        *declared = join_option(*declared, option);
    }

    /// Records the `commodity` directives and the amounts within the `entry`.
    pub fn observe<Deco: Decoration>(&mut self, entry: &LedgerEntry<'_, Deco>) {
        match &entry.statement {
            LedgerStatement::Txn(txn) => {
                for posting in &txn.posts {
                    self.observe_posting(posting.as_undecorated());
                }
            }
            LedgerStatement::Commodity(declaration) => {
                for detail in &declaration.details {
                    if let CommodityDetail::Format(amount) = detail {
                        self.declare_amount(&declaration.name, amount);
                    }
                }
            }
            // `apply tag` may hold an amount, but only as an unparsed string.
            LedgerStatement::Comment(_)
            | LedgerStatement::ApplyTag(_)
            | LedgerStatement::EndApplyTag
            | LedgerStatement::Include(_)
            | LedgerStatement::Account(_) => {}
        }
    }

    /// Returns the [`DisplayContext`] out of everything observed so far.
    pub fn build(self) -> DisplayContext {
        DisplayContext::new(
            self.default_commodity,
            self.commodities
                .into_iter()
                .map(|(commodity, estimate)| {
                    (
                        commodity,
                        CommodityDisplayOption {
                            format: estimate.declared.format.or(estimate.observed.format),
                            min_scale: estimate.declared.min_scale.or(estimate.observed.min_scale),
                        },
                    )
                })
                .collect(),
        )
    }

    fn observe_posting<Deco: Decoration>(&mut self, posting: &Posting<'_, Deco>) {
        if let Some(posting_amount) = &posting.amount {
            // Note the cost and the lot price are not observed on purpose,
            // see the type level comment.
            self.observe_value_expr(posting_amount.amount.as_undecorated());
        }
        if let Some(balance) = &posting.balance {
            self.observe_value_expr(balance.as_undecorated());
        }
    }

    /// Records every amount literal within the `value` expression.
    fn observe_value_expr(&mut self, value: &expr::ValueExpr<'_>) {
        match value.accept(self) {
            Ok(()) => (),
        }
    }

    /// Declares the format given as a `commodity ... format` sample amount.
    ///
    /// The sample carries its own commodity, and that is what the declaration
    /// applies to: the `format` line is printed through the very same context,
    /// so keying it on anything else would make the file grow on every format.
    fn declare_amount(&mut self, declared_name: &str, sample: &expr::Amount<'_>) {
        let commodity = if sample.commodity.is_empty() {
            declared_name
        } else {
            sample.commodity.as_ref()
        };
        let option = as_display_option(&sample.value);
        if let Some(previous) = self.commodities.get(commodity).map(|x| x.declared)
            && (previous.format.is_some() && previous.format != option.format
                || previous.min_scale.is_some() && previous.min_scale != option.min_scale)
        {
            log::warn!(
                "conflicting format declared for the commodity {}: {:?} and {:?}",
                commodity,
                previous,
                option
            );
        }
        self.declare(commodity, option);
    }
}

/// Collects the amount literals out of an expression, ignoring the operators.
///
/// Only [`ExprVisitor::visit_amount`] carries information here, so the operator
/// nodes just recurse and the dispatching nodes keep their default.
impl<'i> expr::ExprVisitor<'i> for DisplayContextBuilder {
    type Output = ();
    type Error = Infallible;

    fn visit_amount(&mut self, amount: &expr::Amount<'i>) -> Result<(), Infallible> {
        // An amount without a commodity is a plain number, `= 0` or an operand
        // in an expression. Formatting those after some unrelated commodity
        // would be nonsense.
        if amount.commodity.is_empty() {
            return Ok(());
        }
        let observed = &mut self
            .commodities
            .entry(amount.commodity.as_ref().to_owned())
            .or_default()
            .observed;
        *observed = join_option(*observed, as_display_option(&amount.value));
        Ok(())
    }

    fn visit_unary(&mut self, expr: &expr::UnaryOpExpr<'i>) -> Result<(), Infallible> {
        self.visit_expr(&expr.expr)
    }

    fn visit_binary(&mut self, expr: &expr::BinaryOpExpr<'i>) -> Result<(), Infallible> {
        self.visit_expr(&expr.lhs)?;
        self.visit_expr(&expr.rhs)
    }
}

/// Returns the display option the given literal implies.
fn as_display_option(value: &PrettyDecimal) -> CommodityDisplayOption {
    CommodityDisplayOption {
        format: value.format,
        // Decimal caps the scale at 28, so the conversion can't fail.
        min_scale: Some(value.scale().try_into().unwrap_or(u8::MAX)),
    }
}

/// Combines the two options so that the result never renders less than either.
fn join_option(x: CommodityDisplayOption, y: CommodityDisplayOption) -> CommodityDisplayOption {
    CommodityDisplayOption {
        format: join_format(x.format, y.format),
        min_scale: std::cmp::max(x.min_scale, y.min_scale),
    }
}

/// Combines the two formats, preferring the one carrying more information.
///
/// `pretty_decimal::Format` is `non_exhaustive`, so an unknown variant is
/// assumed to be more specific than the ones we know.
fn join_format(
    x: Option<pretty_decimal::Format>,
    y: Option<pretty_decimal::Format>,
) -> Option<pretty_decimal::Format> {
    fn rank(format: Option<pretty_decimal::Format>) -> u8 {
        match format {
            None => 0,
            Some(pretty_decimal::Format::Plain) => 1,
            Some(pretty_decimal::Format::Comma3Dot) => 2,
            Some(_) => 3,
        }
    }
    if rank(y) > rank(x) { y } else { x }
}

/// Object combined with the `DisplayContext`.
pub struct WithContext<'a, T> {
    value: &'a T,
    context: &'a DisplayContext,
}

impl<'a, T> WithContext<'a, T> {
    fn pass_context<U>(&self, other: &'a U) -> WithContext<'a, U> {
        WithContext {
            value: other,
            context: self.context,
        }
    }
}

impl<Deco: Decoration> fmt::Display for WithContext<'_, LedgerEntry<'_, Deco>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.value.separation == Separation::BlankLine {
            writeln!(f)?;
        }
        self.pass_context(&self.value.statement).fmt(f)
    }
}

impl<Deco: Decoration> fmt::Display for WithContext<'_, LedgerStatement<'_, Deco>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            LedgerStatement::Txn(txn) => self.pass_context(txn).fmt(f),
            LedgerStatement::Comment(v) => v.fmt(f),
            LedgerStatement::ApplyTag(v) => v.fmt(f),
            LedgerStatement::EndApplyTag => writeln!(f, "end apply tag"),
            LedgerStatement::Include(v) => v.fmt(f),
            LedgerStatement::Account(v) => v.fmt(f),
            LedgerStatement::Commodity(v) => self.pass_context(v).fmt(f),
        }
    }
}

#[derive(Debug)]
struct LineWrapStr<'a> {
    prefix: &'static str,
    content: &'a str,
}

impl<'a> LineWrapStr<'a> {
    fn wrap(prefix: &'static str, content: &'a str) -> Self {
        Self { prefix, content }
    }
}

impl fmt::Display for LineWrapStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in self.content.lines() {
            writeln!(f, "{}{}", self.prefix, line)?;
        }
        Ok(())
    }
}

impl fmt::Display for TopLevelComment<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        LineWrapStr::wrap(";", &self.0).fmt(f)
    }
}

impl fmt::Display for ApplyTag<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "apply tag {}", self.key)?;
        match &self.value {
            None => writeln!(f),
            Some(v) => writeln!(f, "{}", v),
        }
    }
}

impl fmt::Display for IncludeFile<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "include {}", self.0)
    }
}

impl fmt::Display for AccountDeclaration<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "account {}", self.name)?;
        for detail in &self.details {
            detail.fmt(f)?;
        }
        Ok(())
    }
}
impl fmt::Display for AccountDetail<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // The parser keeps the space after the comment prefix as part of the
            // content, so the prefix must not add one back (that would grow by a
            // space on every format).
            AccountDetail::Comment(v) => LineWrapStr::wrap("    ;", v).fmt(f),
            AccountDetail::Note(v) => LineWrapStr::wrap("    note ", v).fmt(f),
            AccountDetail::Alias(v) => writeln!(f, "    alias {}", v),
        }
    }
}

impl fmt::Display for WithContext<'_, CommodityDeclaration<'_>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "commodity {}", self.value.name)?;
        for detail in &self.value.details {
            self.pass_context(detail).fmt(f)?;
        }
        Ok(())
    }
}
impl fmt::Display for WithContext<'_, CommodityDetail<'_>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            // The parser keeps the space after the comment prefix as part of the
            // content, so the prefix must not add one back (that would grow by a
            // space on every format).
            CommodityDetail::Comment(v) => LineWrapStr::wrap("    ;", v).fmt(f),
            CommodityDetail::Note(v) => LineWrapStr::wrap("    note ", v).fmt(f),
            CommodityDetail::Alias(v) => writeln!(f, "    alias {}", v),
            CommodityDetail::Format(v) => writeln!(f, "    format {}", self.pass_context(v)),
        }
    }
}
impl<Deco: Decoration> fmt::Display for WithContext<'_, Transaction<'_, Deco>> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let xact = self.value;
        write!(f, "{}", xact.date.format("%Y/%m/%d"))?;
        if let Some(edate) = &xact.effective_date {
            write!(f, "={}", edate.format("%Y/%m/%d"))?;
        }
        write!(f, " {}", print_clear_state(xact.clear_state))?;
        if let Some(code) = &xact.code {
            write!(f, "({}) ", code)?;
        }
        writeln!(f, "{}", xact.payee)?;
        for m in &xact.metadata {
            m.fmt(f)?;
        }
        for post in &xact.posts {
            write!(f, "{}", self.context.as_display(post.as_undecorated()))?;
        }
        Ok(())
    }
}

const METADATA_PREFIX: &str = "    ; ";

impl fmt::Display for Metadata<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Metadata::WordTags(tags) => {
                f.write_str(METADATA_PREFIX)?;
                f.write_str(":")?;
                for tag in tags {
                    write!(f, "{}:", tag)?;
                }
                f.write_str("\n")?
            }
            Metadata::KeyValueTag { key, value } => {
                f.write_str(METADATA_PREFIX)?;
                writeln!(f, "{}{}", key, value)?
            }
            Metadata::Comment(s) => LineWrapStr::wrap(METADATA_PREFIX, s).fmt(f)?,
        };
        Ok(())
    }
}

impl fmt::Display for MetadataValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetadataValue::Expr(expr) => write!(f, ":: {}", expr),
            MetadataValue::Text(text) => write!(f, ": {}", text),
        }
    }
}

impl<Deco: Decoration> fmt::Display for WithContext<'_, Posting<'_, Deco>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let post = self.value;
        let post_clear = print_clear_state(post.clear_state);
        write!(f, "    {}{}", post_clear, post.account.as_undecorated())?;
        let account_width = UnicodeWidthStr::width_cjk(post.account.as_undecorated().as_ref())
            + UnicodeWidthStr::width(post_clear);
        if let Some(amount) = &post.amount {
            let mut amount_str = String::new();
            let alignment = self
                .pass_context(amount.amount.as_undecorated())
                .fmt_with_alignment(&mut amount_str)?
                .absolute();
            write!(
                f,
                "{:>width$}{}",
                "",
                amount_str.as_str(),
                width = get_column(48, account_width + alignment, 2)
            )?;
            write!(f, "{}", self.pass_context(&amount.lot))?;
            if let Some(exchange) = &amount.cost {
                match exchange.as_undecorated() {
                    Exchange::Rate(v) => write!(f, " @ {}", self.pass_context(v)),
                    Exchange::Total(v) => write!(f, " @@ {}", self.pass_context(v)),
                }?
            }
        }
        if let Some(balance) = &post.balance {
            let mut balance_str = String::new();
            let alignment = self
                .pass_context(balance.as_undecorated())
                .fmt_with_alignment(&mut balance_str)?
                .absolute();
            let trailing = UnicodeWidthStr::width_cjk(balance_str.as_str()) - alignment;
            let balance_padding = if post.amount.is_some() {
                0
            } else {
                get_column(50 + trailing, account_width, 2)
            };
            write!(
                f,
                "{:>width$} {}",
                " =",
                self.pass_context(balance.as_undecorated()),
                width = balance_padding
            )?;
        }
        writeln!(f)?;
        for m in &post.metadata {
            m.fmt(f)?;
        }
        Ok(())
    }
}

impl<Deco: Decoration> fmt::Display for WithContext<'_, Lot<'_, Deco>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(price) = &self.value.price {
            match price.as_undecorated() {
                Exchange::Total(e) => write!(f, " {{{{{}}}}}", self.pass_context(e)),
                Exchange::Rate(e) => write!(f, " {{{}}}", self.pass_context(e)),
            }?;
        }
        if let Some(date) = &self.value.date {
            write!(f, " [{}]", date.format("%Y/%m/%d"))?;
        }
        if let Some(note) = &self.value.note {
            write!(f, " ({})", note)?;
        }
        Ok(())
    }
}

/// Represents a width that should come to the **left** part of each posting.
///
/// Examples:
/// ```text
/// ----*----| <- alignment point
///          v
///         0           // 1
///   (1 + 1)           // 7
///         2 USD       // 1
///        (1 USD * 2)  // 2
///    (2 * 1 USD)      // 6
/// ```
#[derive(Debug, PartialEq, Clone, Copy)]
enum Alignment {
    /// Still alignment wasn't found.
    /// Equivalent to the fact that no commodity is used.
    Partial(usize),
    /// Already alignment was found.
    /// Equivalent to the fact that commodity is used at least once.
    Complete(usize),
}

impl Alignment {
    /// Takes out the width regardless of the alignment type.
    fn absolute(self) -> usize {
        match self {
            Alignment::Complete(x) => x,
            Alignment::Partial(x) => x,
        }
    }

    /// Adds up prefix / suffix lengths.
    fn plus(self, prefix_length: usize, suffix_length: usize) -> Alignment {
        match self {
            Alignment::Partial(x) => Alignment::Partial(prefix_length + x + suffix_length),
            Alignment::Complete(x) => Alignment::Complete(prefix_length + x),
        }
    }
}

trait DisplayWithAlignment {
    fn fmt_with_alignment<W: fmt::Write>(&self, f: &mut W) -> Result<Alignment, fmt::Error>;
}

impl<T> fmt::Display for WithContext<'_, T>
where
    Self: DisplayWithAlignment,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_alignment(f).map(|_| ())
    }
}

/// [`expr::ExprVisitor`] printing the expression, and returning the length until the alignment.
struct ExprFormatter<'a, W> {
    writer: &'a mut W,
    context: &'a DisplayContext,
}

impl<'i, W: fmt::Write> expr::ExprVisitor<'i> for ExprFormatter<'_, W> {
    type Output = Alignment;
    type Error = fmt::Error;

    fn visit_amount(&mut self, amount: &expr::Amount<'i>) -> Result<Alignment, fmt::Error> {
        let amount_str = rescale(amount, self.context).to_string();
        // TODO: Implement prefix-amount.
        if amount.commodity.is_empty() {
            write!(self.writer, "{}", amount_str)?;
            return Ok(Alignment::Partial(amount_str.as_str().len()));
        }
        write!(self.writer, "{} {}", amount_str, amount.commodity)?;
        // Given the amount is only [0-9.], it's ok to count bytes.
        Ok(Alignment::Complete(amount_str.as_str().len()))
    }

    fn visit_value_expr(&mut self, expr: &expr::ValueExpr<'i>) -> Result<Alignment, fmt::Error> {
        match expr {
            expr::ValueExpr::Amount(a) => self.visit_amount(a),
            expr::ValueExpr::Paren(inner) => {
                self.writer.write_char('(')?;
                let alignment = self.visit_expr(inner)?;
                self.writer.write_char(')')?;
                Ok(alignment.plus(1, 1))
            }
        }
    }

    fn visit_unary(&mut self, expr: &expr::UnaryOpExpr<'i>) -> Result<Alignment, fmt::Error> {
        write!(self.writer, "{}", expr.op)?;
        self.visit_expr(&expr.expr).map(|x| x.plus(1, 0))
    }

    fn visit_binary(&mut self, expr: &expr::BinaryOpExpr<'i>) -> Result<Alignment, fmt::Error> {
        let a1 = self.visit_expr(&expr.lhs)?;
        write!(self.writer, " {} ", expr.op)?;
        let a2 = self.visit_expr(&expr.rhs)?;
        Ok(match a1.plus(0, 3) {
            Alignment::Complete(x) => Alignment::Complete(x),
            Alignment::Partial(x) => a2.plus(x, 0),
        })
    }
}

macro_rules! display_with_alignment_by_visitor {
    ($t:ty) => {
        impl DisplayWithAlignment for WithContext<'_, $t> {
            fn fmt_with_alignment<W: fmt::Write>(
                &self,
                f: &mut W,
            ) -> Result<Alignment, fmt::Error> {
                self.value.accept(&mut ExprFormatter {
                    writer: f,
                    context: self.context,
                })
            }
        }
    };
}

display_with_alignment_by_visitor!(expr::ValueExpr<'_>);
display_with_alignment_by_visitor!(expr::Amount<'_>);

/// Returns column shift size so that the string will be located at `colsize`.
/// At least `padding` is guaranteed to be spaced.
fn get_column(colsize: usize, left: usize, padding: usize) -> usize {
    if left + padding < colsize {
        colsize - left
    } else {
        padding
    }
}

fn rescale(x: &expr::Amount, context: &DisplayContext) -> PrettyDecimal {
    let mut v = x.value;
    if let Some(min_scale) = context.min_scale(x.commodity.as_ref()) {
        v.as_mut().normalize_assign();
        v.rescale(std::cmp::max(min_scale.into(), v.scale()));
    }
    match context.decimal_format(x.commodity.as_ref()) {
        Some(format) => PrettyDecimal::with_format(v.value, Some(format)),
        None => v,
    }
}

fn print_clear_state(v: ClearState) -> &'static str {
    match v {
        ClearState::Uncleared => "",
        ClearState::Cleared => "* ",
        ClearState::Pending => "! ",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    use maplit::hashmap;
    use pretty_assertions::assert_eq;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    fn amount<'a, T, U>(value: T, commodity: U) -> expr::ValueExpr<'a>
    where
        T: Into<Decimal>,
        U: Into<Cow<'a, str>>,
    {
        let value: Decimal = value.into();
        expr::ValueExpr::Amount(expr::Amount {
            commodity: commodity.into(),
            value: PrettyDecimal::unformatted(value),
        })
    }

    fn amount_expr<T: Into<Decimal>>(value: T, commodity: &'static str) -> expr::Expr<'static> {
        let value: Decimal = value.into();
        expr::Expr::Value(Box::new(amount(value, commodity)))
    }

    #[test]
    fn display_ledger_entries_no_txn() {
        let ctx = DisplayContext::default();
        assert_eq!(
            concat!(";this\n", ";is\n", ";a pen pineapple apple pen.\n"),
            format!(
                "{}",
                ctx.as_display(&plain::LedgerStatement::Comment(TopLevelComment(
                    Cow::Borrowed("this\nis\na pen pineapple apple pen."),
                )))
            )
        );
        assert_eq!(
            "apply tag foo\n",
            format!(
                "{}",
                ctx.as_display(&tracked::LedgerStatement::ApplyTag(ApplyTag {
                    key: Cow::Borrowed("foo"),
                    value: None
                })),
            )
        );
        assert_eq!(
            "apply tag foo: bar\n",
            format!(
                "{}",
                ctx.as_display(&plain::LedgerStatement::ApplyTag(ApplyTag {
                    key: Cow::Borrowed("foo"),
                    value: Some(MetadataValue::Text(Cow::Borrowed("bar")))
                }))
            ),
        );
        assert_eq!(
            "apply tag foo:: 100\n",
            format!(
                "{}",
                ctx.as_display(&tracked::LedgerStatement::ApplyTag(ApplyTag {
                    key: Cow::Borrowed("foo"),
                    value: Some(MetadataValue::Expr(Cow::Borrowed("100")))
                }))
            ),
        );
        assert_eq!(
            "end apply tag\n",
            format!("{}", ctx.as_display(&plain::LedgerStatement::EndApplyTag))
        );
    }

    #[test]
    fn display_ledger_entry_emits_blank_line_on_separation() {
        let ctx = DisplayContext::default();
        let entry = |separation| plain::LedgerEntry {
            separation,
            statement: plain::LedgerStatement::EndApplyTag,
        };
        assert_eq!(
            "end apply tag\n",
            format!("{}", ctx.as_display(&entry(Separation::Immediate)))
        );
        assert_eq!(
            "\nend apply tag\n",
            format!("{}", ctx.as_display(&entry(Separation::BlankLine)))
        );
    }

    #[test]
    fn display_txn() {
        let got = format!(
            "{}",
            DisplayContext::default().as_display(&LedgerStatement::Txn(plain::Transaction {
                date: NaiveDate::from_ymd_opt(2022, 12, 23).unwrap(),
                effective_date: None,
                clear_state: ClearState::Uncleared,
                code: None,
                payee: Cow::Borrowed("Example Grocery"),
                posts: vec![Posting {
                    account: Cow::Borrowed("Assets"),
                    clear_state: ClearState::Uncleared,
                    amount: Some(PostingAmount {
                        amount: amount(dec!(123.45), "USD"),
                        cost: None,
                        lot: Lot::default(),
                    }),
                    balance: None,
                    metadata: vec![Metadata::Comment("single-line".into()),],
                }],
                metadata: vec![
                    Metadata::Comment("multi\nline\ntext\n".into()),
                    Metadata::Comment("works\nfine".into()),
                ],
            }))
        );
        let want = concat!(
            "2022/12/23 Example Grocery\n",
            "    ; multi\n",
            "    ; line\n",
            "    ; text\n",
            "    ; works\n",
            "    ; fine\n",
            "    Assets                                    123.45 USD\n",
            "    ; single-line\n"
        );
        assert_eq!(want, got);
    }

    #[test]
    fn posting_non_expr() {
        let all = Posting {
            amount: Some(PostingAmount {
                amount: amount(1234, "USD"),
                cost: Some(Exchange::Rate(amount(dec!(100.00), "JPY"))),
                lot: plain::Lot {
                    price: Some(Exchange::Rate(amount(dec!(1.1), "USD"))),
                    date: Some(NaiveDate::from_ymd_opt(2022, 5, 20).unwrap()),
                    note: Some(Cow::Borrowed("printable note")),
                },
            }),
            balance: Some(amount(1234, "USD")),
            ..Posting::new_untracked("Account")
        };
        let costbalance = Posting {
            amount: Some(PostingAmount {
                amount: amount(1234, "USD"),
                cost: Some(Exchange::Rate(amount(100, "JPY"))),
                lot: plain::Lot::default(),
            }),
            balance: Some(amount(1234, "USD")),
            ..Posting::new_untracked("Account")
        };
        let total = Posting {
            amount: Some(PostingAmount {
                amount: amount(1234, "USD"),
                cost: Some(Exchange::Total(amount(100, "JPY"))),
                lot: plain::Lot::default(),
            }),
            ..Posting::new_untracked("Account")
        };
        let nocost = Posting {
            amount: Some(PostingAmount {
                amount: amount(1234, "USD"),
                cost: None,
                lot: plain::Lot::default(),
            }),
            balance: Some(amount(1234, "USD")),
            ..Posting::new_untracked("Account")
        };
        let noamount = plain::Posting {
            amount: None,
            balance: Some(amount(1234, "USD")),
            ..Posting::new_untracked("Account")
        };
        let zerobalance = plain::Posting {
            amount: None,
            balance: Some(amount(0, "")),
            ..Posting::new_untracked("Account")
        };

        assert_eq!(
            concat!(
                //       10        20        30        40        50        60        70
                // 34567890123456789012345678901234567890123456789012345678901234567890
                "    Account                                     1234 USD {1.1 USD} [2022/05/20] (printable note) @ 100.00 JPY = 1234 USD\n",
                "    Account                                     1234 USD @ 100 JPY = 1234 USD\n",
                "    Account                                     1234 USD @@ 100 JPY\n",
                "    Account                                     1234 USD = 1234 USD\n",
                "    Account                                              = 1234 USD\n",
                // we don't have shared state to determine where = should be aligned
                "    Account                                          = 0\n"
            ),
            format!(
                "{}{}{}{}{}{}",
                DisplayContext::default().as_display(&all),
                DisplayContext::default().as_display(&costbalance),
                DisplayContext::default().as_display(&total),
                DisplayContext::default().as_display(&nocost),
                DisplayContext::default().as_display(&noamount),
                DisplayContext::default().as_display(&zerobalance),
            ),
        );

        // overrides only
        let ctx = DisplayContext::new(
            CommodityDisplayOption::default(),
            hashmap! {"USD".to_string() => CommodityDisplayOption {format: Some(pretty_decimal::Format::Comma3Dot), min_scale: Some(4)}},
        );
        assert_eq!(
            concat!(
                //       10        20        30        40        50        60        70
                // 34567890123456789012345678901234567890123456789012345678901234567890
                "    Account                               1,234.0000 USD {1.1000 USD} [2022/05/20] (printable note) @ 100.00 JPY = 1,234.0000 USD\n",
                "    Account                               1,234.0000 USD @ 100 JPY = 1,234.0000 USD\n",
                "    Account                               1,234.0000 USD @@ 100 JPY\n",
                "    Account                               1,234.0000 USD = 1,234.0000 USD\n",
                "    Account                                              = 1,234.0000 USD\n",
                "    Account                                          = 0\n"
            ),
            format!(
                "{}{}{}{}{}{}",
                ctx.as_display(&all),
                ctx.as_display(&costbalance),
                ctx.as_display(&total),
                ctx.as_display(&nocost),
                ctx.as_display(&noamount),
                ctx.as_display(&zerobalance),
            ),
        );
    }

    #[test]
    fn fmt_posting_comma_3_dot() {
        let ctx = DisplayContext::default();
        let large = plain::Posting {
            amount: Some(
                expr::ValueExpr::Amount(expr::Amount {
                    commodity: Cow::Borrowed("JPY"),
                    value: PrettyDecimal::comma3dot(dec!(1_234_567)),
                })
                .into(),
            ),
            ..Posting::new_untracked("Account")
        };
        let small = plain::Posting {
            amount: Some(
                expr::ValueExpr::Amount(expr::Amount {
                    commodity: Cow::Borrowed("JPY"),
                    value: PrettyDecimal::comma3dot(dec!(0.0011)),
                })
                .into(),
            ),
            ..Posting::new_untracked("Account")
        };

        assert_eq!(
            concat!(
                //       10        20        30        40        50        60        70
                // 34567890123456789012345678901234567890123456789012345678901234567890
                "    Account                                1,234,567 JPY\n",
                "    Account                                   0.0011 JPY\n",
            ),
            format!("{}{}", ctx.as_display(&large), ctx.as_display(&small),),
        );
    }

    #[test]
    fn fmt_with_alignment_simple_amount_without_commodity() {
        let mut buffer = String::new();
        let alignment = DisplayContext::default()
            .as_display(&amount(123i8, ""))
            .fmt_with_alignment(&mut buffer)
            .unwrap();
        assert_eq!("123", buffer.as_str());
        assert_eq!(Alignment::Partial(3), alignment);
    }

    #[test]
    fn fmt_with_alignment_simple_amount_with_commodity() {
        // no format, no min_scale
        let mut buffer = String::new();
        let usd1234 = amount(1234i16, "USD");
        let alignment = DisplayContext::default()
            .as_display(&usd1234)
            .fmt_with_alignment(&mut buffer)
            .unwrap();
        assert_eq!("1234 USD", buffer.as_str());
        assert_eq!(Alignment::Complete(4), alignment);

        // min_scale
        buffer.clear();
        let alignment = DisplayContext::new(
            CommodityDisplayOption {
                format: None,
                min_scale: Some(2),
            },
            HashMap::new(),
        )
        .as_display(&usd1234)
        .fmt_with_alignment(&mut buffer)
        .unwrap();
        assert_eq!("1234.00 USD", buffer.as_str());
        assert_eq!(Alignment::Complete(7), alignment);

        buffer.clear();
        let alignment = DisplayContext::new(
            CommodityDisplayOption {
                format: Some(pretty_decimal::Format::Comma3Dot),
                min_scale: Some(2),
            },
            HashMap::new(),
        )
        .as_display(&usd1234)
        .fmt_with_alignment(&mut buffer)
        .unwrap();
        assert_eq!("1,234.00 USD", buffer.as_str());
        assert_eq!(Alignment::Complete(8), alignment);
    }

    #[test]
    fn test_fmt_with_alignment_complex_expr() {
        // ((1.20 + 2.67) * 3.1 USD + 5 USD)
        let expr = expr::ValueExpr::Paren(expr::Expr::Binary(expr::BinaryOpExpr {
            lhs: Box::new(expr::Expr::Binary(expr::BinaryOpExpr {
                lhs: Box::new(expr::Expr::Value(Box::new(expr::ValueExpr::Paren(
                    expr::Expr::Binary(expr::BinaryOpExpr {
                        lhs: Box::new(amount_expr(dec!(1.20), "")),
                        op: expr::BinaryOp::Add,
                        rhs: Box::new(amount_expr(dec!(2.67), "")),
                    }),
                )))),
                op: expr::BinaryOp::Mul,
                rhs: Box::new(amount_expr(dec!(3.1), "USD")),
            })),
            op: expr::BinaryOp::Add,
            rhs: Box::new(amount_expr(5i32, "USD")),
        }));
        let mut got = String::new();
        let alignment = DisplayContext::default()
            .as_display(&expr)
            .fmt_with_alignment(&mut got)
            .unwrap();
        assert_eq!("((1.20 + 2.67) * 3.1 USD + 5 USD)", got.as_str());
        assert_eq!(Alignment::Complete(20), alignment);
    }
}

#[cfg(test)]
mod display_context_builder_tests {
    use super::*;

    use crate::parse::{self, ParseOptions};

    use pretty_assertions::assert_eq;

    /// Returns the context derived out of the given Ledger content.
    fn context_of(input: &str) -> DisplayContext {
        let mut builder = DisplayContextBuilder::new();
        for parsed in parse::parse_ledger::<plain::Ident>(&ParseOptions::default(), input) {
            let (_, entry) = parsed.expect("test input must be parsed");
            builder.observe(&entry);
        }
        builder.build()
    }

    /// Returns the option resolved for the `commodity`.
    fn option_of(context: &DisplayContext, commodity: &str) -> CommodityDisplayOption {
        CommodityDisplayOption {
            format: context.decimal_format(commodity),
            min_scale: context.min_scale(commodity),
        }
    }

    fn option(format: Option<pretty_decimal::Format>, min_scale: u8) -> CommodityDisplayOption {
        CommodityDisplayOption {
            format,
            min_scale: Some(min_scale),
        }
    }

    #[test]
    fn observe_takes_the_widest_scale_of_the_literals() {
        let context = context_of(indoc::indoc! {"
            2024/01/01 Widen
                Expenses:Grocery       1 CHF
                Expenses:Household     2.345 CHF
                Assets:Bank           -3.34 CHF
        "});

        assert_eq!(option(None, 3), option_of(&context, "CHF"));
    }

    #[test]
    fn observe_prefers_the_style_carrying_more_information() {
        let context = context_of(indoc::indoc! {"
            2024/01/01 Mixed
                Expenses:Grocery       1234.00 CHF
                Expenses:Household     5,678.00 CHF
                Assets:Bank           -6912.00 CHF
        "});

        assert_eq!(
            option(Some(pretty_decimal::Format::Comma3Dot), 2),
            option_of(&context, "CHF")
        );
    }

    #[test]
    fn observe_skips_the_amount_without_commodity() {
        let context = context_of(indoc::indoc! {"
            2024/01/01 No commodity
                Expenses:Grocery       (-10 * 2.100)
                Assets:Bank            = 0
        "});

        assert_eq!(CommodityDisplayOption::default(), option_of(&context, ""));
        assert_eq!(
            CommodityDisplayOption::default(),
            option_of(&context, "CHF")
        );
    }

    #[test]
    fn observe_takes_the_balance_but_not_the_cost_nor_the_lot_price() {
        let context = context_of(indoc::indoc! {"
            2024/01/01 Exchange
                Assets:Broker          2 SPINX {1.2345 USD} @ 1.6789 USD
                Assets:Bank           -3.3 USD = 1234.56 USD
        "});

        // Neither the lot price nor the cost widens USD beyond the balance.
        assert_eq!(
            option(Some(pretty_decimal::Format::Plain), 2),
            option_of(&context, "USD")
        );
        assert_eq!(option(None, 0), option_of(&context, "SPINX"));
    }

    #[test]
    fn observe_walks_into_the_expressions() {
        let context = context_of(indoc::indoc! {"
            2024/01/01 Expression
                Expenses:Grocery       (-(1.20 EUR) + 2.6700 EUR * 2)
                Assets:Bank            -4.14 EUR
        "});

        assert_eq!(option(None, 4), option_of(&context, "EUR"));
    }

    #[test]
    fn observe_prefers_the_declared_format_over_the_observed_one() {
        let context = context_of(indoc::indoc! {"
            commodity CHF
                format 1,000.00 CHF

            2024/01/01 Precise
                Expenses:Grocery       1.23456 CHF
                Assets:Bank           -1.23456 CHF
        "});

        assert_eq!(
            option(Some(pretty_decimal::Format::Comma3Dot), 2),
            option_of(&context, "CHF")
        );
    }

    #[test]
    fn observe_binds_the_declaration_to_the_sample_commodity() {
        // A weird declaration, but the `format` line is printed through the
        // very same context, so it must be keyed on the sample's commodity.
        let context = context_of(indoc::indoc! {"
            commodity CHF
                format 1,000.00 USD
        "});

        assert_eq!(
            option(Some(pretty_decimal::Format::Comma3Dot), 2),
            option_of(&context, "USD")
        );
        assert_eq!(
            CommodityDisplayOption::default(),
            option_of(&context, "CHF")
        );
    }

    #[test]
    fn observe_joins_the_declarations_regardless_of_the_order() {
        let plain_first = context_of(indoc::indoc! {"
            commodity CHF
                format 1000.00 CHF

            commodity CHF
                format 1,000.000 CHF
        "});
        let comma_first = context_of(indoc::indoc! {"
            commodity CHF
                format 1,000.000 CHF

            commodity CHF
                format 1000.00 CHF
        "});

        let want = option(Some(pretty_decimal::Format::Comma3Dot), 3);
        assert_eq!(want, option_of(&plain_first, "CHF"));
        assert_eq!(want, option_of(&comma_first, "CHF"));
    }

    #[test]
    fn declare_overrides_what_is_observed() {
        let mut builder = DisplayContextBuilder::new();
        for parsed in parse::parse_ledger::<plain::Ident>(
            &ParseOptions::default(),
            "2024/01/01 Observed\n    Assets:Bank  1.234 JPY\n",
        ) {
            let (_, entry) = parsed.expect("test input must be parsed");
            builder.observe(&entry);
        }
        builder.declare("JPY", option(Some(pretty_decimal::Format::Comma3Dot), 0));

        assert_eq!(
            option(Some(pretty_decimal::Format::Comma3Dot), 0),
            option_of(&builder.build(), "JPY")
        );
    }

    #[test]
    fn default_commodity_applies_to_the_unseen_commodity() {
        let builder = DisplayContextBuilder::new()
            .with_default_commodity(option(Some(pretty_decimal::Format::Comma3Dot), 2));

        assert_eq!(
            option(Some(pretty_decimal::Format::Comma3Dot), 2),
            option_of(&builder.build(), "CHF")
        );
    }
}
