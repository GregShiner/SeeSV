# Design
## Parsing
### Pre-process
Find all quotation mark and newline using simd in seperate thread, with offsets stored in (2) arrays
1. `quote_offsets = [...]`
2. `newline_offsets = [...]`

> [!NOTE]
> Because the scan is linear, all of the offsets will be in ascending order.

### Find chunk boundaries
1. Split the files into multiple chunks of X bytes, in pratice this just mean coming up with a list of multiples of X bytes, i.e `[offset_start_chunk1, offset_start_chunk2, ...]`
  - These will be used as the inital guesses for a chunk boundary
2. Take `offset_start_chunk`, binary search `newline_offsets` for the nearest newline
3. Binary search `quote_offsets` for the right nearest to the position of the newline
4. If the index of the quote is odd (in 0 based, an odd index mean even parity of quotes) then the newline is valid
5. Else the newline is within a quotation mark, repeat step 3 with next newline.
6. output an array `[start1, start2, ....]` until the final chunk, these starting point will be valid newlines.

### Chunk parsing

Multiple threads each processing one chunk at a time

Vaguely: Finite state machine scan through the chunk and parse according to this [csv spec](https://www.rfc-editor.org/rfc/rfc4180)

TODO: the rest

## Model View

After parsing is complete the data will be fragmented inside memories, in order to expose a singular array that the query engine can use, 
a `view` (which is just an array of pointers) need to be make.

The `view`:
1. An array of fat pointers (address, and length) the address will be pointed to the raw memory region where the data lives.
2. Can encapsulate multiple region of memories, or multiple partial region of memories 

For example: Given 4 fragment of memories, totaling 99 bytes, with 3 views, the views would be split evenly 33 bytes each. 
Since the fragments of memories won't be evenly distributed, the views would contain the start and length of multiple fragments.

<img src="https://media.discordapp.net/attachments/1240874027602673774/1450707387550797955/image.png?ex=6943840a&is=6942328a&hm=c1248e53462268ad946c6e7069334664e087e4d8cf8ab461b8d95c8d56f10fe2&=&format=webp&quality=lossless&width=1036&height=785">

## Defragging In the background

A zig process can run in the background to consolidate fragmented data into continous arrays (this is done in the background to avoid upfront expensive memcpy)

1. Defrag in the background and then swap out the fat pointers in the `view`, this can be done safely at any time.
2. Free old memory, this need to make sure the old memory isn't in use.

### Epoch counter
Epoch counter can be use to track the version of `views` that are in use, if the minium version of the `views` in use are higher than the old memory, it can be safely free. 

TODO

## ABI

TODO
