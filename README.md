# npm_time_machine

```
NPM Time Machine - Move package.json through the time!

Usage: npm_time_machine [OPTIONS] <DATE>

Arguments:
  <DATE>  Target date (format: DD-MM-YYYY)

Options:
  -f <INPUT_FILE>       input file [default: package.json]
  -o <OUTPUT_FILE>      output file [default: package.json.out]
      --no-cache        Don't use / reload cache
      --silent          Silent mode
      --dry-run         Dry run - show changes only
  -h, --help            Print help (see more with '--help')
```

## Description

`npm_time_machine` is a `package.json` point-in-time "pinner". Given the date it'll compare `package.json` pin with latest stable version for a library and use the newer one.

Intended use is to bisect dates to find point where upgrade span can be managed, e.g. by avoiding scenarios where multiple core libraries undergo change.

### Installation

`cargo install npm_time_machine`

## Resolution example

`package.json`:
- Package A - version 1.0.0
- Package B - version 1.0.0

State at: `01-01-2020`
- Package A - Version 0.5.0
- Package B - Version 1.1.0

Resolved state for `01-01-2020`:
- Package A: Version 1.0.0 (*`package.json` has newer than target date*)
- Package B: Version 1.1.0 (*target date has newer version*)

## Rationale

- Node-backed projects have tendency to develop network of dependencies between libraries
- Popular libraries introduce major versions every ~1-3 years requiring code rewrite, sometimes extensive
- Skipping upgrades in front-end projects might be viable option
- With passing time, upgrade gets harder and harder as libraries are interconnected and upgrade of one might pull other ones
- When projects are big enough and still developing, it's:
  - impossible to stop development process for upgrade
  - difficult to ensure safety and stability (especially since test libraries can also be subject to upgrade)
  - neigh impossible to have PR reviewable

Reasonable, agile solution is to split upgrades into multiple "checkpoints".

Each checkpoint is small and stable enough that it can be merged into the code-base and full upgrade can be worked over course of months without disrupting progress.

*As I hadn't found the tool to do it I decided to write my own.*

## Demo
![](./extras/vhs_demo/demo.mov)

## Caveats / known issues

- [ ] time machine cache is created in `cwd` - **might pollute dirtree**
- [ ] `devDependencies` aren't processed
- [ ] only single date format is supported
- [ ] due to limited test input there might be handling blind spots
- [ ] not tested on pinned non-stable versions (RCs, alphas, betas etc.)
