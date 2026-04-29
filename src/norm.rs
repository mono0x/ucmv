use unicode_normalization::UnicodeNormalization;

#[derive(Copy, Clone, Debug)]
pub enum Form {
    Nfc,
    Nfd,
}

pub fn convert(name: &str, form: Form) -> String {
    match form {
        Form::Nfc => name.nfc().collect(),
        Form::Nfd => name.nfd().collect(),
    }
}
