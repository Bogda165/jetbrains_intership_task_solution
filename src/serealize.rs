pub trait Serialize {
    fn serialize(self) -> Vec<u8>
    where
        Self: Sized;
}

pub trait Deserialize {
    fn desrialize(buffer: Vec<u8>) -> Result<Self, String>
    where
        Self: Sized;
}
