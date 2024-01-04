# One Billion Rows

This is my attempt at wrangling [The One Billion Rows Challenge](https://github.com/gunnarmorling/1brc/tree/main).

## Projects

* `create_measurements` is a direct port of the Java program that builds the 14gb measurements file. It's slow (you could easily speed it up by using a better random number generator). I didn't bother optimizing this, it just creates the input data and won't be used often. On my workstation, it takes about 109 seconds to run.
* `naieve_create_average` is a direct port of the original `CalculateAverage` project from the Java repo. This is meant to serve as a baseline. It takes about 97 seconds on my workstation, which is quite awful.
* `having_fun` is my attempt to go quite fast. See below. It averages 3.2 seconds on my workstation.

## Having Fun

The `having_fun` project is IO bound on my computer. Above my physical core count, I get exactly the same result each time. That means there's probably room for improvement, but not on my PC!

I start by using the `memmap2` crate to memory map the target file. That lets me treat it as a big slice of 
`u8`, and lets Linux handle the "fun" part of paging it in and out.

Once the file is memory mapped, I divide it into roughly equal-sized chunks by dividing the buffer size by the number of CPUs.
For each section that isn't the first, I scan ahead until I find a `\n` (newline) - ensuring that each chunk starts on a
station boundary.

Next, I use a set of scoped threads for each chunk (scoped to make it easy to share the memory map). For each chunk,
the reader reads one byte at a time. If its a `:`, we know that's the end of the name. If its a `\n`, that's the end of the entry.
That gives us everything we need to have slices (references inside the master slice) to the bytes making up each portion
of the data.

Each thread maintains its own `HashMap`, keyed on a slice of bytes (the name). It contains minimum, maximum,
count and sum fields---and a `String` containing the name. For each record, if an entry doesn't exist
we create one---otherwise we update the fields. We only go through the hoops of converting the name
if the aggregator doesn't contain one---so we skip that overhead except for the first time we encounter
a station.

Finally, we aggregate the results, sort and print them out.

### Updates

* Updated to use `ahash` instead of the built-in `HashMap` and now it is hitting 3.2 seconds!





