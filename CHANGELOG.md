# 0.7.0
- Make `Mobi::readable_records_range` public - it returns the range of PDB records that contain the books content
- Remove `Mobi::content` method.
- Add `Mobi::raw_records` that returns a wrapper over parsed raw PDB records with slices to their content.
- Make Palmdoc lz77 decompression work
- Add `first_index_record` field to `MobiHeader`
- `Mobi::exth_record`, `Mobi::exth_record_at`, `ExthHeader::get_record` now return a `Option<&Vec<Vec<u8>>>` instead of `Option<&Vec<u8>>` because some records like subject can occur multiple times. 
- Add `MobiMetadata::subjects` that returns a list of subjects.
