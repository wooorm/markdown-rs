# Contribute

> 👉 **Important**:
> this project has a [code of conduct][coc].
> By interacting with this repository and community you agree to abide by its
> terms.

This article explains how to contribute.
Please read through the following guidelines.

## Contributions

There are several ways to contribute, not just by writing code.
See [Support][] if you have questions.

### Financial support

You can help financially.
See [Sponsor][] for more info.

### Improve docs

As a user you’re perfect to help improve the docs.
Typo corrections, error fixes, better explanations, new examples, etcetera.

### Improve issues

Some issues lack information, aren’t reproducible, or are just incorrect.
You can help by trying to make them easier to resolve.
Existing issues might benefit from your unique experience or opinions.

### Write code

Code contributions are very welcome too.
It’s probably a good idea to first post a question or open an issue to report a
bug or suggest a new feature before creating a pull request.
See [Project][] for more info.

## Submitting an issue

* the issue tracker is for issues, discussions are for questions
* search the issue tracker (including closed issues) before opening a new
  issue
* ensure you’re using the latest versions of packages and other tools
* use a clear and descriptive title
* include as much information as possible: steps to reproduce the issue,
  error message, version, operating system, etcetera
* the more time you put into an issue, the better help you can get
* the best issue report is a failing test] proving it

## Submitting a pull request

* run `cargo fmt` and `cargo test` locally to format and test your changes
* non-trivial changes are often best discussed in an issue first, to prevent
  you from doing unnecessary work
* for ambitious tasks, you should try to get your work in front of the
  community for feedback as soon as possible
* new features should be accompanied by tests and documentation
* don’t include unrelated changes
* write a convincing description of why your pull request should land:
  it’s your job to be convincing

## Project (for maintainers)

See [Project][project] in the readme for info on how the project is structured
and how to run useful scripts.

### Release

* update the `version` field in `Cargo.toml`
* search for the previous version in `readme.md`, replace with new one
* `git commit --all --message 1.2.3 && git tag 1.2.3 && git push && git push --tags`
* `cargo publish`

(similar for `mdast_util_to_markdown`)

## Resources

* [how to contribute to open source](https://opensource.guide/how-to-contribute/)
* [making your first contribution](https://medium.com/@vadimdemedes/making-your-first-contribution-de6576ddb190)
* [using pull requests](https://help.github.com/articles/about-pull-requests/)
* [GitHub help](https://help.github.com)

## License

[CC-BY-4.0][license] © [Titus Wormer][author]

<!-- Definitions -->

[license]: https://creativecommons.org/licenses/by/4.0/

[author]: https://wooorm.com

[support]: support.md

[coc]: code-of-conduct.md

[sponsor]: https://github.com/wooorm/markdown-rs/#sponsor

[project]: https://github.com/wooorm/markdown-rs/#project
