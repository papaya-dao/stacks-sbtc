# Contributing to core-eng

Welocme to the core-eng repo!
This repo is primarily used by the Core Engineering team at TrustMachines, but anyone is welcome to contribute.
This document provides guidelines and instructions to help make the contribution process smooth and efficient.

## Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md).
Please report any unacceptable behavior to [core-eng-leads@trustmachines.co](mailto:core-eng-leads@trustmachines.co) or any other project maintainer.

## Issues

When submitting an issue, please adhere to the following best practices:

1. **Search for existing issues**: Before creating a new issue, search the [existing issues](https://github.com/Trust-Machines/core-eng/issues) to avoid duplicates. If a similar issue exists, contribute to the discussion there instead of opening a new one.
2. **Provide a clear and concise title**: Write a meaningful title that reflects the issue's content. This helps maintainers and contributors quickly grasp the issue's context.
3. **Add a definition of done**: Provide a clear criteria for when the issue is concidered done. This ensures that the implementation lives up to the original intention of the issue.
4. **Include necessary details**: Explain the issue thoroughly, including steps to reproduce (if applicable), expected behavior, and encountered problems. Include relevant screenshots or log files to better illustrate the issue.
5. **Stay on topic**: Keep the discussion focused on the issue at hand. Avoid discussing unrelated topics or providing unnecessary information.

## Pull Requests

When creating a pull request, please follow these guidelines to ensure a smooth and efficient review process:

1. **Build on the main branch**: We don't maintain any other branches than `main`. All changes should be integrated into `main` as early as possible.
2. **Write a descriptive title**: Summarize your changes with a clear and informative title, highlighting the purpose of the pull request.
3. **Link to a related issue**: Each pull request should address one and only one issue. Reference the issue number in the description (e.g., `Resolves #123` or `Closes #123`). Trivial hotfixes may be exempt from this rule, but please mention this in the description.
4. **Explain your changes**: Describe the changes you made and the reasoning behind them. Explain how the changes satisfies the definition of done of the addressed issue.
5. **Address review feedback**: Respond to the comments and feedback provided by reviewers. Make any necessary changes and update your pull request accordingly. Resolve any open discussions before merging.

By following these guidelines, you'll help maintain a productive and collaborative environment for everyone involved in the project. Thank you for contributing!

## Developer Certificate of Origin (DCO)

To certify that you have the necessary rights to submit your work and that you agree to the terms of the project's license,
you must [sign off](https://git-scm.com/docs/git-commit#Documentation/git-commit.txt---signoff) your commits.

This is done by appending the following to your commit messages:

```
Signed-off-by: Your Name <youremail@example.com>
```

Using the git CLI this can be achieved with the `--signoff` flag or `-s` with the `git commit` command:

```
git commit -s -m "Your commit message here"
```

By signing off your commits, you agree to the [Developer Certificate of Origin](DCO.md):

Thank you for contributing. May the force be with you!