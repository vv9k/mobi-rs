# 0.7.0
- Make `Mobi::readable_records_range` public - it returns the range of PDB records that contain the books content
- Remove `Mobi::content` method.
- Add `Mobi::raw_records` that returns a wrapper over parsed raw PDB records with slices to their content.
