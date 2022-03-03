use std::env;
use std::fs;
use std::fs::File;
use std::io::*;
use std::str;

struct PngChunk {
    chunk_length: [u8;4],
    chunk_type: [u8;4], 
    chunk_data: Vec<u8>,
    chunk_crc: [u8;4]
}

impl PngChunk{
    fn get_header(&self) -> String{
        match str::from_utf8(&self.chunk_type) {
            Ok(v) => v.to_string(),
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        }
    }

    fn get_crc(&self) -> u32{
        unsafe { std::mem::transmute::<[u8; 4], u32>(self.chunk_crc) }.to_le()
    }

    fn get_length(&self) -> u32{
        let mut chunk_sz = [0u8;4];
        chunk_sz.clone_from_slice(&self.chunk_length);
        chunk_sz.reverse();
        unsafe { std::mem::transmute::<[u8; 4], u32>(chunk_sz) }.to_le()
    }

    fn create_chunk(header: String, data: String) -> PngChunk {
        let mut header2 = [0u8;4];
        header2.clone_from_slice(header.as_bytes());
        let data = data.as_bytes().to_vec() ;
        let mut len = unsafe { std::mem::transmute::<u32, [u8; 4]>(data.len() as u32) };
        len.reverse();
        PngChunk {
            chunk_length: len,
            chunk_type: header2,
            chunk_data: data,
            chunk_crc: [0,0,0,0]
        }
    }
}
struct PngFile{
    file_signature: [u8;8],
    file_content: Vec<PngChunk>
}

impl PngFile {
    fn get_last_chunk(&self) -> &PngChunk{
        self.file_content.get(self.file_content.len()-1).expect("File is empty")
    }

    fn get_chunk(&self, index: usize) -> &PngChunk{
        self.file_content.get(index).expect("Index out of range!")
    }

    fn insert_chunk(&mut self, chunk: PngChunk, index: usize){
        self.file_content.insert(index, chunk);
    }

    fn save(&self, filename: String, inj_chunk: PngChunk, inj_after: String){
        let mut file = File::create(filename).unwrap(); 
        
        file.write(&self.file_signature).unwrap();
        for chunk in &self.file_content{
            file.write(&chunk.chunk_length).unwrap();
            
            file.write(&chunk.chunk_type).unwrap();
            
            file.write(&chunk.chunk_data).unwrap();
            file.write(&chunk.chunk_crc).unwrap();
            if chunk.get_header() == inj_after{
                file.write(&inj_chunk.chunk_length).unwrap();
                write!(&mut file, "{}", str::from_utf8(&inj_chunk.chunk_type).unwrap().to_string()).unwrap();
                write!(&mut file, "{}", str::from_utf8(&inj_chunk.chunk_data).unwrap().to_string()).unwrap();
                file.write(& inj_chunk.chunk_crc).unwrap();
            }
        }
    }

    fn create() -> PngFile{
        PngFile {
            file_signature: [137,80, 78, 71, 13, 10, 26, 10],
            file_content: Vec::<PngChunk>::new()
        }
    }
}

fn check_if_png(mut file:&File) -> bool{
    let mut buffer = [0u8;8];
    file.read(&mut buffer).unwrap();
    buffer==[137,80, 78, 71, 13, 10, 26, 10]
}

fn read_chunk(mut file: &File) -> PngChunk {
    let mut chunk = PngChunk {
        chunk_length:[0u8;4],
        chunk_type: [0u8;4],
        chunk_data: [0u8;8].to_vec(),
        chunk_crc: [0u8;4]
    };

    // READING CHUNK LENGTH
    file.read(&mut chunk.chunk_length).unwrap();
    
    //READING CHUNK HEADER
    file.read(&mut chunk.chunk_type).unwrap();
    
    //READING CHUNK DATA
    chunk.chunk_data=vec![0u8;chunk.get_length() as usize];
    file.read_exact(&mut chunk.chunk_data).unwrap();

    //READING CHUNK CRC
    file.read(&mut chunk.chunk_crc).unwrap();

    chunk
}

fn read_png(file: &File) -> PngFile{
    let mut pngfile = PngFile::create();
    let mut quit = false;
    while !quit {
        let chunk = read_chunk(file);
        pngfile.insert_chunk(chunk, pngfile.file_content.len());
        if pngfile.get_last_chunk().get_header() == "IEND"{
            quit = true;
        }
    }
    pngfile
}
fn main() {
    let argv:Vec<String> = env::args().collect();
    if argv.len()!=2{
        println!("Wrong amount of arguments: expected 1, got {}", argv.len()-1);
        return;
    }
    let filename = argv.get(1).unwrap();
    let file = fs::File::open(filename).unwrap();
    if check_if_png(&file){
        println!("{} is png image", filename);
    }
    else {
        println!("{} is not png image", filename);    
    }
    let mut png = read_png(&file);
    for chunk in &png.file_content{
        println!("Chunk header: {:?}", chunk.get_header());
        println!("Chunk size:   {}"  , chunk.get_length());
        println!("Chunk CRC:    0x{}", format!("{:x}", chunk.get_crc()));
        println!("-------------------------------------");
    }
    let data = "someusefulinfo".to_string();
    let header= "keKW".to_string();
    let inj_chunk = PngChunk::create_chunk( header, data);
    png.save("kekw.png".to_string(), inj_chunk, "IHDR".to_string());
    
}
