use validator::Validate;

pub trait Validator<T: Validate = Self>: Validate {
    fn parse_error_codes(&self) -> Option<Vec<String>> {
        let mut v = vec![];

        match self.validate() {
            Ok(_v) => None,
            Err(e) => {
                let errors = e.field_errors();

                for key in errors.keys() {
                    let errors = errors.get(key).unwrap();

                    for i in 0..errors.len() {
                        let message = errors[i].message.as_deref();

                        match message {
                            Some(m) => v.push(m.to_string()),
                            None => v.push(format!("{}_required", key)),
                        }
                    }
                }

                match v.len() {
                    0 => None,
                    _ => Some(v),
                }
            }
        }
    }
}
