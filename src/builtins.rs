use crate::object::{BuiltinFunction, Error, Object, Integer};


pub fn len(args: Vec<Object>) -> Object{
    if args.len() != 1 {
        return Object::Error(Error::wrong_number_of_parms(args.len() as i64, 1));
    }

    if let Object::String_(string) = &args[0] {
        return Object::Integer(Integer{value: string.value.len() as i64});

    } else {
        return Object::Error(Error::type_has_no_method(args[0].object_type(), "len()"));
    }
}

pub fn builtins(func: &str) -> Option<BuiltinFunction> {
    match func {
        "len" => Some(len),
        _ =>None 
    }
}
