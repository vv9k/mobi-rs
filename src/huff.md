# Decompressing Huffman Records in Mobi Files #

Bit and byte indexes begin at zero, and refer to the most significant bit or byte first. Mobipocket files, and all values stored internally, are stored in big endian format. 

For instance, bit 0 of 0b10 refers to 1, and bit 1 refers to 0. Byte 0 of 0xF0 refers to F, and byte 1 refers to 0.

## Huffman Table  ##
The Huffman table consists of the following:
1. **Huffman Dictionary**: A mapping of indices to slices of compressed or decompressed bytes
2. **Code Dictionary**: A fixed size mapping from all 8 bit values (0 to 255), to a 3-tuple consisting of a code length which can be a value from 1 to 31, inclusive, a boolean term, and a maximum code, which is an unsigned 32 bit value
3. **Minimum and Maximum Codes**: A mapping from values 1 to 31, inclusive, to an unsigned 32 bit minimum code, and an unsigned 32 bit maximum code 

Its structure in file is:
- bytes 0-4: "HUFF" as utf8 bytes
- bytes 4-8: 0x00_00_00_18
- bytes 8-12: Offset to the **code dictionary (2.)**
- bytes 12-16: Offset to **minimum and maximum codes (3.)**

When parsing the individual records, the first record refers to this Huffman table, and all other records are CDIC records.

### Code Dictionary
The code dictionary consists of 256 distinct values, each 4 bytes in length, starting at the specified offset. The 3-tuples are indexed, starting from 0, in order of appearance.

Each value is as follows:
- bits 0-24: A value which represents the maximum code
- bit 24: A boolean flag
- bits 25-27: Unknown
- bits 27-32: A code length

The code length must not be zero, and the maximum possible code length is 31. 

If the code length is 8 or less, the boolean flag must be set.

The maximum code is derived as follows:
``` ((max_code_value + 1) << (32 - code_length)) - 1 ```

### Minimum and Maximum Codes
The code dictionary consists of 32 pairs of minimum and maximum codes, with each pair being 8 bytes.
In each pair, the minimum code representation appears in the four most significant bytes, and the maximum code representation appears in the four least significant bytes. 

The values are indexed, starting from 1, in order of appearance. This index can be thought of as the code length.

The minimum code is derived as follows:
``` min_code_value << (32 - index) ```

The maximum code is derived as follows:
``` ((max_code_value + 1) << (32 - index)) - 1 ```

Note that since the maximum code is 32 bits, the last minimum code is 0, and the last maximum code is 0xFFFF_FFFF.

### Huffman Dictionary
The dictionary is initialized by individually collecting the byte slices from all CDIC records in order of appearance.

#### CDIC Record
When reading CDIC records, the dictionary must also be provided.

A CDIC record is stored as follows:
- bytes 0-4: "CDIC" as utf8 bytes
- bytes 4-8: 0x00_00_00_10
- bytes 8-12: The number of "phrases"
- bytes 12-16: A value used to calculate the maximum number of phrases.

To determine the number of slices available in the record, use the following calculation:

```n = minimum(pow(2, max_number_of_phrases), num_phrases - size(dictionary))```

Starting at byte 16, **n** 16 bit offsets, (with respect to the start of the CDIC record) are available.

The first two bytes at a given offset refer to a flag and a length, as follows:
- bit 0: The compression flag.
- bits 1-16: The slice length in number of bytes.

If the compression flag is set, the slice will require decompression later. Otherwise, it can be used verbatim. The slice starts at the third byte, immediately after the flag and length, and is **length** bytes long.

## Decompression
When decompressing, the size of the Huffman dictionary does not change, but individual slices may be decompressed. 

The procedure to decompress either data or internal byte slices is as follows:
```
fn decompress(data: Bytes, *) -> Result<Bytes, Error> {
    data_bits <- data as bits
    bit_index <- 0
    decompressed <- []
    
    while bit_index < size(data_bits) {
        code <- data_bits[bit_index..bit_index+32]
        
        (code_length, term, maximum_code) <- code_dictionary[code[0..8]]
    
        if not term {
            # code_length will always exist, since the last minimum_code is 0.
            code_length <- code_length + index(
                minimum_codes[code_length..],
                fn cmp(minimum_code) { code >= minimum_code }
            )
            maximum_code <- maximum_codes[code_length]
        }
        
        index <- (maximum_code - code)[0..code_length]
        if not exists(huffman_dictionary[index]) {
            return InvalidIndex;
        }
        
        slice <- huffman_dictionary[index]
        
        if compressed(slice) {
            pop(huffman_dictionary[index])
            slice <- decompress(slice, *)
            insert(huffman_dictionary, index, slice)
        }
        
        decompressed <- decompressed + slice
        
        bit_index <- bit_index + code_length
    }
    
    decompressed
}
```
Note that indexing into data_bits is done from most significant to least significant bit (and if a given bit index is not available, it should be substituted with zeros), and that * refers to all values which appear in the function body, but are not explicitly passed in (refer to previous sections to find what these values refer to).
