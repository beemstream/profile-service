use validator::Validate;
use rocket_contrib::json;
use rocket_contrib::json::JsonValue;

pub trait Validator<T: Validate = Self>: Validate {

    fn parsed_field_errors(&self) -> Option<Vec<JsonValue>> {
        match self.validate() {
            Ok(_v) => None,
            Err(e) => {
                let errors = e.field_errors();

                let mut parsed_errors = vec![];

                for key in errors.keys() {
                    let errors = errors.get(key).unwrap();

                    let mut v = vec![];
                    for i in 0..errors.len() {
                        v.push(&errors[i].message);
                    }
                    parsed_errors.push(json!({ "name": key, "message": v }));
                }

                if parsed_errors.len() > 0 {
                    Some(parsed_errors)
                } else {
                    None
                }

            }
        }
    }
}


