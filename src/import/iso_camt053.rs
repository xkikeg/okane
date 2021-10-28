mod xmlnode;


use quick_xml::de::DeError;

fn parse_camt<R: std::io::BufRead>(reader: &mut R) -> Result<xmlnode::Document, DeError> {
    let document: xmlnode::Document = quick_xml::de::from_reader(reader)?;
    Ok(document)
}

// Debug only function.
pub fn print_camt<R: std::io::BufRead>(reader: &mut R) -> Result<String, DeError> {
    let res = parse_camt(reader)?;
    return Ok(format!("{:#?}", res));
}
