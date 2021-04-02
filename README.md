# ArgonOne Fan Service

ArgonOne case for Raspberry Pi 4B fan service.

```bash
./build.sh
sudo argonfand -g
sudo argonfand -f 100 # enforce max speed and exit
sudo argonfand # start service
```

### Service

```bash
systemctl status argonfand.service
systemctl start argonfand.service
systemctl enable argonfand.service
```
