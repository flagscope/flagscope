> Status: approved
> Last updated: 2026-11-26
> Mode: new-project

# Decisions: lighter-scans

## AD-19: Read light-mode symbol files from memory, and download distinct symbol files in parallel

Date: 2026-11-26
Context: HU-09 makes light mode the default (RF-16.1, RF-16.2). On 2026-11-26, symbol files built from only the blocks the scanner reads gave the same features as complete files, and Microsoft's symbol server answered HTTP range requests. The first full scans took 51 to 58 minutes downloading one file at a time.
Alternatives considered:
- Write each partial symbol file to disk with the missing blocks left empty: simple, but on NTFS a file extended to its full size takes that space unless it is marked sparse, so light mode would use as much disk as full mode.
- Implement an MSF reader of our own for the symbol record stream: avoids depending on how `ms-pdb` reads, but duplicates format code `ms-pdb` already has (AD-05).
- Keep downloading one file at a time: no threads, but most of a scan is spent waiting on request round trips (about 0.4 s each) rather than transferring.
Decision: In light mode, the scanner first reads each symbol file's MSF header, stream directory, and debug information header with a few HTTP range requests, then downloads the PDB information stream and the symbol record stream, requesting consecutive blocks together. The blocks are held in memory, and `ms-pdb` reads them through its `ReadAt` input (blocks never downloaded read as zeros). Files that are not MSF 7.0, or whose structure is inconsistent, are downloaded whole. Binaries that reference the same symbol file (same name, GUID, and age) share one download and one result. Planning runs on 16 threads and downloading on 8 threads (`std::thread::scope`, as AD-07 allows); the numbers are to be tuned with the timings of the verification batch.
Consequences: Light mode writes nothing to disk; its memory use is bounded by eight symbol record streams at a time (the largest measured is about 56 MB). In light mode the estimate is shown after the file structures are read, so a small download happens before it; the estimate reports that amount separately. Results reach the observer in the order downloads finish, not in file order. On a sample of four system binaries plus one duplicate, light mode downloaded 74 MB instead of 386 MB, in 28 s instead of 107 s, and produced the same 556 features; the duplicate was not downloaded twice.
Status: active
