use validator::Validate;
use super::response::FieldError;

pub trait Validator<T: Validate = Self>: Validate {

    fn parsed_field_errors(&self) -> Option<Vec<FieldError>> {
        match self.validate() {
            Ok(_v) => None,
            Err(e) => {
                let errors = e.field_errors();
                let mut parsed_errors = vec![];

                for key in errors.keys() {
                    let errors = errors.get(key).unwrap();

                    let mut v = vec![];
                    for i in 0..errors.len() {
                        let message = errors[i].message.as_deref().unwrap();
                        v.push(message.to_string());
                    }
                    parsed_errors.push(FieldError::new(String::from(*key), v));
                }

                match parsed_errors.len() {
                    0 => None,
                    _ => Some(parsed_errors)
                }
            }
        }
    }
}

