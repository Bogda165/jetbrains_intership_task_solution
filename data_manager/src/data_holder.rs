use std::error::Error;

pub trait DataHolderError: Error {}

pub trait DataHolder {
    type DataType;
    type DataContainer: IntoIterator<Item = Self::DataType>;
    type E: DataHolderError;

    fn request(&mut self, bounds: (usize, usize)) -> Result<(), Self::E>;
    fn get_response(&mut self) -> Result<Option<(Self::DataContainer, (usize, usize))>, Self::E>;
    fn get_data_len(&self) -> usize;
}
