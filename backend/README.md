# Backend for Kunjika

## Quiickstart

Create a `template.json` file from the sample. Also, create a `.env` file for SQLx and use
migrations to create the database.

Then `cargo build && cargo watch -x 'run -- -v 2 -c template.json'` will get the backend up and
running.

## Details

This is backend for Kunjika, a minimal QA or forum. This backend is written using `Rust + ntex`
developed by Nikolay Kim. I have been programming for 19 years. First 10 years I was a C/C++
systems programmer but I was never happy with C++ because I could see that there are many
things which make life complicated in it for no reason.

Rust as a programming language is a Godsend. I do not see any reason to use any other programming
language. Similarly, in web frameworks `ntex` is superior in performance to most other frameworks.
The API is clean and elegant. The server runs in few tens of megabytes of RAM, which is very sweet.

The main work of art in Stackoverflow is its tag-engine which I will probably never implement,
but then again what do I know of future.

I will add features slowly as I get time to work on this project. Though it is a minimal
QA/Forum, I will not shy away from adding useful features.