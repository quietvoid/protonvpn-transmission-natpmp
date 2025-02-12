## protonvpn-transmission-natpmp

Daemon to enable port forwarding with ProtonVPN to use with a remote Transmission client.

The program does the following:
- Every 50s, a NATPMP request is sent for both TCP/UDP ports.
- The forwarded port is verified against the running remote Transmission client.  
  If the port is different, Transmission's peer port is updated to the new port.

Every request and change is logged to file.

### Configuration

- Copy the template config file to `.protonvpn-transmission-natpmp.cfg`.  
  The file must be present in the same directory as the executable.
- Set the Transmission RPC client credentials.

### systemd service

A basic `systemd` service would look like this:

```
[Unit]
Description=protonvpn-transmission-natpmp

[Service]
Type=simple
KillMode=mixed

Restart=on-failure
RestartSec=5

WorkingDirectory=/path/to/protonvpn-transmission-natpmp
ExecStart=/path/to/protonvpn-transmission-natpmp/protonvpn-transmission-natpmp

[Install]
WantedBy=multi-user.target
```
