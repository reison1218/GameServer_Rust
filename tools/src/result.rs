use super::*;
use errors::*;
pub mod feature {
    error_chain! {}
}

pub mod inner {
    error_chain! {}
}
pub mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {

        links {
            Inner(crate::result::inner::Error, crate::result::inner::ErrorKind) #[doc = "Doc"];
            // Attributes can be added at the end of the declaration.
            Feature(crate::result::feature::Error, crate::result::feature::ErrorKind) #[doc = "Doc"];
        }
        foreign_links {
            Io(::std::io::Error) #[doc = "Error during IO"];
            //None(std::option::NoneError) #[doc = "Error during NoneError"];

            Protobuf(protobuf::error::ProtobufError) #[doc = "Error during Protobuf"];

            Send(std::sync::mpsc::SendError<crate::tcp::Data>) #[doc = "Error during Send"];
        }
        errors {
            Single {
                description("MyError!")
                display("Single Error")
            }
            Duple(t: String) {
                description("MyError!")
                display("Dutple {} Error", t)
            }
            Multi(len: u32, data: Vec<u32>) {
                description("MyError!")
                display("Multi len {} data {:?} Error", len, data)
            }
        }
    }
}