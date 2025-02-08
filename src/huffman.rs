use std::{
    collections::{BinaryHeap, HashMap},
    io::Read,
};

#[derive(Eq)]
pub struct MinHeapNode {
    data: Option<u8>,
    freq: u32,
    left: Option<Box<MinHeapNode>>,
    right: Option<Box<MinHeapNode>>,
}

impl MinHeapNode {
    pub fn new(data: Option<u8>, freq: u32) -> Self {
        Self {
            data,
            freq,
            left: None,
            right: None,
        }
    }
}

impl PartialEq for MinHeapNode {
    fn eq(&self, other: &Self) -> bool {
        self.freq == other.freq
    }
}

impl Ord for MinHeapNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.freq.cmp(&self.freq)
    }
}

impl PartialOrd for MinHeapNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(other.freq.cmp(&self.freq))
    }
}

pub fn lookup_table(
    root: Option<Box<MinHeapNode>>,
    code: Vec<u8>,
    table: &mut HashMap<u8, Vec<u8>>,
) {
    let Some(root) = root else {
        return;
    };

    if let Some(data) = root.data {
        table.insert(data, code.clone());
    }

    if let Some(left) = root.left {
        let mut left_code = code.clone();
        left_code.push(0);
        lookup_table(Some(left), left_code, table);
    }

    if let Some(right) = root.right {
        let mut right_code = code;
        right_code.push(1);
        lookup_table(Some(right), right_code, table);
    }
}

fn huffman_table(data: Vec<u8>, freq: &[u32]) -> BinaryHeap<MinHeapNode> {
    let mut min_heap = BinaryHeap::new();

    for i in 0..data.len() {
        min_heap.push(MinHeapNode::new(
            data.get(i).cloned(),
            *freq.get(i).unwrap(),
        ));
    }

    while min_heap.len() != 1 {
        let left = min_heap.pop().unwrap();
        let right = min_heap.pop().unwrap();

        let mut top = MinHeapNode::new(None, left.freq + right.freq);

        top.left = Some(Box::new(left));
        top.right = Some(Box::new(right));

        min_heap.push(top);
    }

    min_heap
}

pub fn get_frequencies(data: &[u8]) -> (Vec<u8>, Vec<u32>) {
    let mut freq: HashMap<u8, u32> = HashMap::new();

    for val in data {
        *freq.entry(*val).or_insert(0) += 1;
    }

    let mut freq_vec: Vec<(u8, u32)> = freq.into_iter().collect();
    freq_vec.sort_by(|a, b| b.1.cmp(&a.1));

    let (sorted_chars, sorted_freqs): (Vec<u8>, Vec<u32>) =
        freq_vec.iter().map(|(c, f)| (*c, *f)).unzip();

    (sorted_chars, sorted_freqs)
}

pub fn huffman_encode(buffer: &[u8]) -> Vec<u8> {
    let (arr, freq) = get_frequencies(&buffer);
    let mut heap = huffman_table(arr, &freq);
    let mut table = HashMap::new();
    lookup_table(heap.pop().map(|v| Box::new(v)), Vec::new(), &mut table);

    let mut table_bytes = Vec::new();
    for (char, code) in &table {
        table_bytes.push(*char as u8);
        table_bytes.push(code.len() as u8);

        let mut packed_code: u16 = 0;
        for (i, bit) in code.iter().enumerate() {
            if *bit == 1 {
                packed_code |= 1 << (15 - i);
            }
        }
        table_bytes.extend_from_slice(&packed_code.to_le_bytes());
    }

    let mut bit_stream: Vec<u8> = Vec::new();
    for char in buffer {
        let code = table.get(&char).unwrap();
        bit_stream.extend(code);
    }

    let mut buffer: Vec<u8> = Vec::new();
    let mut byte = 0u8;
    let mut bit_count = 0;

    for bit in bit_stream {
        byte |= bit << (7 - bit_count);
        bit_count += 1;

        if bit_count == 8 {
            buffer.push(byte);
            byte = 0;
            bit_count = 0;
        }
    }

    // If there are remaining bits, pad them to make a full byte
    if bit_count > 0 {
        buffer.push(byte);
    }

    let mut new_buffer: Vec<u8> = Vec::new();
    new_buffer.extend_from_slice(&(table_bytes.len() as u16).to_le_bytes());
    new_buffer.extend_from_slice(&table_bytes);
    new_buffer.extend_from_slice(&buffer);

    new_buffer
}

pub fn huffman_decode(buffer: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut cursor = std::io::Cursor::new(buffer);

    let mut table_size_len = u16::MAX.to_le_bytes();
    cursor.read_exact(&mut table_size_len)?;
    let table_size = u16::from_le_bytes(table_size_len) as usize;

    let mut table = HashMap::new();

    while (cursor.position() as usize) < table_size + table_size_len.len() {
        let mut char_byte = [0u8; 1];
        cursor.read_exact(&mut char_byte)?;
        let char_byte = char_byte[0];

        let mut packed_code_len = [0u8; 1];
        cursor.read_exact(&mut packed_code_len)?;
        let packed_code_len = u8::from_le_bytes(packed_code_len);

        let mut packed_code_bytes = [0u8; 2];
        cursor.read_exact(&mut packed_code_bytes)?;
        let packed_code = u16::from_le_bytes(packed_code_bytes);

        let mut code = Vec::new();
        for i in 0..packed_code_len {
            let bit = (packed_code >> (15 - i)) & 1;
            code.push(bit as u8);
        }

        table.insert(code, char_byte);
    }

    let mut decoded_bytes = Vec::new();
    let mut bit_stream = Vec::new();

    let remaining_data = cursor.get_ref();
    for &byte in &remaining_data[cursor.position() as usize..] {
        for i in (0..8).rev() {
            bit_stream.push(if (byte >> i) & 1 == 1 { 1 } else { 0 });
        }
    }

    let mut current_code = Vec::new();
    for bit in bit_stream {
        current_code.push(bit);
        if let Some(&byte) = table.get(&current_code) {
            decoded_bytes.push(byte);
            current_code.clear();
        }
    }

    Ok(decoded_bytes)
}
