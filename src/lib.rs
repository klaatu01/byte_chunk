use bytes::Bytes;
use std::fmt::Display;

pub struct ByteChunks<'a, T: 'a> {
    v: &'a [T],
    chunk_byte_size: usize,
}

pub trait SizeInBytes {
    fn bytes_size(&self) -> usize;
}

impl SizeInBytes for String {
    fn bytes_size(&self) -> usize {
        let bytes: Bytes = self.to_owned().into();
        bytes.len()
    }
}

impl<'a> SizeInBytes for &'a str {
    fn bytes_size(&self) -> usize {
        let bytes: Bytes = self.to_string().into();
        bytes.len()
    }
}

impl<'a, T: 'a> ByteChunks<'a, T>
where
    T: SizeInBytes,
{
    pub fn new(slice: &'a [T], size: usize) -> Self {
        Self {
            v: slice,
            chunk_byte_size: size,
        }
    }

    fn next_split_index(&mut self) -> usize {
        let mut byte_count = 0;
        let mut index = 0;
        loop {
            let next = self.v.get(index);
            match next {
                Some(d) => {
                    let size_of_next = d.bytes_size();
                    if size_of_next > self.chunk_byte_size {
                        panic!("Chunk is larger than {} bytes", self.chunk_byte_size);
                    } else if byte_count + size_of_next > self.chunk_byte_size {
                        break;
                    } else {
                        byte_count += size_of_next;
                        index += 1;
                    }
                }
                None => break,
            }
        }
        index
    }
}

impl<'a, T> Iterator for ByteChunks<'a, T>
where
    T: SizeInBytes,
{
    type Item = &'a [T];

    fn next(&mut self) -> Option<&'a [T]> {
        if self.v.is_empty() {
            None
        } else {
            let chunksz = self.next_split_index();
            let (fst, snd) = self.v.split_at(chunksz);
            self.v = snd;
            Some(fst)
        }
    }
}

pub trait ByteChunked<'a, T> {
    fn byte_chunks(&self, chunk_byte_size: usize) -> ByteChunks<'_, T>;
}

pub trait SafeByteChunkedMut<'a, T> {
    fn byte_chunks_safe_mut(&mut self, chunk_byte_size: usize) -> ByteChunks<'_, T>;
}

pub trait SafeByteChunked<'a, T> {
    fn byte_chunks_safe(&mut self, chunk_byte_size: usize) -> ByteChunks<'_, T>;
}

impl<T> ByteChunked<'_, T> for [T]
where
    T: SizeInBytes,
{
    fn byte_chunks(&self, chunk_byte_size: usize) -> ByteChunks<'_, T> {
        ByteChunks::new(self, chunk_byte_size)
    }
}

impl<T> ByteChunked<'_, T> for Vec<T>
where
    T: SizeInBytes,
{
    fn byte_chunks(&self, chunk_byte_size: usize) -> ByteChunks<'_, T> {
        ByteChunks::new(self.as_slice(), chunk_byte_size)
    }
}

impl<T> SafeByteChunkedMut<'_, T> for Vec<T>
where
    T: SizeInBytes,
{
    fn byte_chunks_safe_mut(&mut self, chunk_byte_size: usize) -> ByteChunks<'_, T> {
        self.retain(|x| x.bytes_size() <= chunk_byte_size);
        ByteChunks::new(self.as_slice(), chunk_byte_size)
    }
}

#[cfg(test)]
mod tests {
    use super::{ByteChunked, SafeByteChunkedMut};
    use std::mem::size_of_val;

    #[test]
    fn test_next_split_index() {
        let data: Vec<&str> = vec!["Hello", "There", "Best", "Worl", "D", "A"];
        let mut chunked = data.as_slice().byte_chunks(10);
        let next_index = chunked.next_split_index();

        assert_eq!(2, next_index);
    }

    #[test]
    fn size_of_tests() {
        let string_ref = "test";
        assert_eq!(size_of_val(string_ref), 4);
    }

    #[test]
    fn creates_chunks_static_str() {
        let data: Vec<&str> = vec!["Hello", "There", "Best", "Worl", "D", "A"];
        let mut chunk_iter = data.byte_chunks(10);
        if let Some(next) = chunk_iter.next() {
            println!("{:?}", next);
            assert_eq!(2, next.len());
        }

        if let Some(next) = chunk_iter.next() {
            println!("{:?}", next);
            assert_eq!(4, next.len());
        }

        let next = chunk_iter.next();
        assert_eq!(None, next);
    }

    #[test]
    fn creates_chunks_string() {
        let data: Vec<String> = (vec!["Hello", "There", "Best", "Worl", "D", "A"])
            .iter()
            .map(|&x| String::from(x))
            .collect();

        let mut chunk_iter = data.byte_chunks(10);

        if let Some(next) = chunk_iter.next() {
            println!("{:?}", next);
            assert_eq!(2, next.len());
        }

        if let Some(next) = chunk_iter.next() {
            println!("{:?}", next);
            assert_eq!(4, next.len());
        }
        let next = chunk_iter.next();
        assert_eq!(None, next);
    }

    #[test]
    fn empty_vec_returns_none() {
        let data: Vec<String> = Vec::new();

        let mut chunk_iter = data.byte_chunks(10);

        let next = chunk_iter.next();
        assert_eq!(None, next);
    }

    //"ラウトは難しいです！" == 30 bytes
    #[test]
    fn special_chars_are_sized_with_string() {
        let data: Vec<String> = vec!["ラウ", "トは", "難し", "いで", "す！"]
            .iter()
            .map(|&x| String::from(x))
            .collect();

        let mut chunk_iter = data.byte_chunks(12);

        if let Some(next) = chunk_iter.next() {
            println!("{:?}", next);
            assert_eq!(2, next.len());
        }

        if let Some(next) = chunk_iter.next() {
            println!("{:?}", next);
            assert_eq!(2, next.len());
        }

        if let Some(next) = chunk_iter.next() {
            println!("{:?}", next);
            assert_eq!(1, next.len());
        }
        let next = chunk_iter.next();
        assert_eq!(None, next);
    }

    #[test]
    fn special_chars_are_sized_with_static_string() {
        let data = vec!["ラウ", "トは", "難し", "いで", "す！"];

        let mut chunk_iter = data.byte_chunks(12);

        if let Some(next) = chunk_iter.next() {
            println!("{:?}", next);
            assert_eq!(2, next.len());
        }

        if let Some(next) = chunk_iter.next() {
            println!("{:?}", next);
            assert_eq!(2, next.len());
        }

        if let Some(next) = chunk_iter.next() {
            println!("{:?}", next);
            assert_eq!(1, next.len());
        }
        let next = chunk_iter.next();
        assert_eq!(None, next);
    }

    #[test]
    fn strings_that_are_too_large_are_skipped() {
        let mut data: Vec<String> = vec!["Hello", "There"]
            .iter()
            .map(|&x| String::from(x))
            .collect();
        let mut chunk_iter = data.byte_chunks_safe_mut(3);

        let next = chunk_iter.next();
        assert_eq!(None, next);
    }
}
