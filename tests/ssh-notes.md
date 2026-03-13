# Skittle Sandbox — SSH & Git Setup

## Setup (one-time)

Make sure your keys are loaded into the macOS agent:

```bash
ssh-add -l                        # check what's loaded
ssh-add ~/.ssh/github_personal    # personal GitHub
ssh-add ~/.ssh/github_adobe       # Adobe GitHub
ssh-add ~/.ssh/git.corp           # Adobe corp git
```

macOS Keychain usually handles this automatically if you've used these keys before. If you see your keys in `ssh-add -l`, skip this step.

## Launch the sandbox

```bash
./tests/sandbox-ssh
```

This:
1. Builds a Docker image with skittle compiled and on PATH
2. Mounts `~/.ssh` and `~/.gitconfig` read-only into the container
3. Starts an SSH server on port 2222

## Connect

```bash
ssh -o StrictHostKeyChecking=no -p 2222 root@localhost
# password: skittle
```

## Inside the container

Your full SSH config and git config are available, so all your host aliases work:

```bash
# Personal repos (github_personal key)
git clone git@github.com:trentdavies/some-repo.git
skittle source add git@github.com:trentdavies/some-repo.git

# Adobe repos (github_adobe key, via URL rewrite in .gitconfig)
git clone git@github.com:OneAdobe/some-repo.git

# Corp git (git.corp key)
git clone git@git.corp.adobe.com:org/repo.git
```

## How it works

The sandbox mounts `~/.ssh` and `~/.gitconfig` as read-only volumes. Your SSH config maps each GitHub host alias to a specific key:

| Key                | SSH Host             | Used for                     |
|--------------------|----------------------|------------------------------|
| `github_personal`  | `github.com`         | `trentdavies/*` repos        |
| `github_adobe`     | `github-adobe`       | Adobe orgs via URL rewrite   |
| `github_enterprise`| `ghe`                | GitHub Enterprise            |
| `git.corp`         | `git.corp.adobe.com` | Adobe corp git               |

Your `.gitconfig` rewrites Adobe org URLs (`OneAdobe`, `tdavies_adobe`, `Adobe-Experience-Platform`) to use the `github-adobe` host alias, so the correct key is selected automatically.

Keys are read-only — nothing inside the container can modify them.

## Cleanup

```bash
docker rm -f skittle-sandbox
```

## Custom port

```bash
./tests/sandbox-ssh 3333
ssh -o StrictHostKeyChecking=no -p 3333 root@localhost
```
