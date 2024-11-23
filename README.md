# fssh 🐟

A SSH config searching tool built with Rust

## Usage

`fssh` will parse your SSH config found at `~/.ssh/config`

Every `Host` will then be shown which also has a corresponding `HostName` entry.

You can then use the arrow keys or `Ctrl+P` and `Ctrl+N` to move the selected entry up or down
or type to filter the entries shown.

Once the Host you want to connect to is selected press `Enter` to connect.

This will then execute `ssh <host>`
E.g if your ssh config looks like this
```
Host my-host
    HostName my-host.hello.com
```

And `my-host      my-host.hello.com` is selected then `ssh my-host` will be executed.

## Future Improvements

* Forwarding arguments to the SSH command
* Use Fuzzy Searching instead of substrings
* Use caching to make searching faster
* Various UI improvements
