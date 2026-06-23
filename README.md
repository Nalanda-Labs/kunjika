# kunjika

## What is kunjika?
Kunjika is a minimal QA/forum application built using Rust, ntex, Svelte, YugabyeDB(changed
from CockroachDB because SQLx 0.9 does not work well with Cockroachdb ),
and Redis. Demo at https://kunjika.ashtavakra.org.

## Is it usable?
It is under active development and is being used in production. Look at demo site

## Is there a binary release for installation?
There is no binary release nor there is any plan to do so.
The installation must be done using source code. Perhaps later I would release a docker
config.

## Installation
Refer to individual READMEs in both backend and frontend directory.


## Why another QA?
It is not about QA/forum only. Most web software is written using inefficient languages
like PHP, Python, Java, Ruby and so on. It is an attempt to develop a web application
in Rust which is very efficient and resource-friendly.

Discourse is very nice forum software but I find it bloated with features but for most
use cases it should work nicely. If you want a lean/mean QA/forum only then you should
choose Kunjika. The other reason is the license. Kunjika is true free software under
GNU GPL and not under some other open source license.

There are some software projects being done in Golang for efficiency. However, I think
that the philosophy behind development of Golang is horribly wrong. Rust is the
silver bullet of programming.

If you want more features you can try Apache Answer written in Golang.

*Kunjika will not integrate with any closed-source/non-GPL applications. Please do not create
issues for such integrations.*
