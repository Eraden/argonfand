# ArgonOne Fan Service

ArgonOne case for Raspberry Pi 4B fan service.

```bash
./build.sh
sudo argonfand -g # generate default config, depends on case location it should be adjusted
sudo argonfand -f 100 # enforce max speed and exit
sudo argonfand # start service
```

### Service

```bash
systemctl status argonfand.service
systemctl start argonfand.service
systemctl enable argonfand.service
```

### Config

Config file must exists before starting application and is located:

`/etc/argonfand.conf`

Format:

```toml
# TEMP=SPEED
49=0
50=40
60=100
```
