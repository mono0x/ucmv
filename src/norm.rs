use unicode_normalization::UnicodeNormalization;

pub enum Form {
    Nfc,
    Nfd,
}

pub fn convert(name: &str, form: &Form) -> String {
    match form {
        Form::Nfc => name.nfc().collect(),
        Form::Nfd => name.nfd().collect(),
    }
}
