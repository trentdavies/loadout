# Tasks

fix these bugs. Run `just test` and `just sandbox-test` after each, and insure all pass. Commit everything. Work in separate worktrees when appropriate. 

then create this feature.

## Bugs

- [ ] Fix this bug: loadout list ‘cl*sk*' - the issue is if you have a skill identity like 'claude-plugins:agent-skills/my-foo-skill'. Then 'cl*sk*' should match it, but it doesn't appear to
- [ ] `bundle list <bundle>` should show the list of skills
- [ ] `status` should show more info
- [ ] `agent list` should show more info - follow the patterns in other commands. colorize. provide more info.


## Feature

Enable an experimental feature, an fzf thing, that is used with `list` subcommands. It allows you to filter the full output of teh command, and will also show a preview on the right, of the relevant skill.md. you see the skills on the left and you can see a preview on the right.

