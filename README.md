<h1 align="center">gl</h1>

## Description

A simple command line utility to wrap some common Git functions into a simple binary.

## Quick Start

```bash
$ ./compile.sh
$ ./gl -h
```

## Configure for yourself

This tool is very much made for myself.  It started as a Bash alias when I first started programming, in August, 2019, and then turned into a [Bash script](https://github.com/jakewilliami/scripts/tree/master/bash/gl), and then [a small Rust project](https://github.com/jakewilliami/scripts/tree/master/rust/gl/), and now this.  While I never intended this tool to be used by others, I figured I should allow some customisability if anybody else wants to use it.

Anything you need to change to get it working for you should be in the [config file](./src/config.rs).

## Where to store

Once it is ready for a "release", I like to store this in `/opt/local/bin`:
```bash
$ ./compile.sh
$ chmod 755 ./gl
$ mv ./gl /opt/local/bin
```

