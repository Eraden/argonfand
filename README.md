# ArgonOne Fan Service

ArgonOne case for Raspberry Pi 4B fan service.

```bash
./build.sh
sudo argonfand -g # generate default config, depends on case location it should be adjusted
sudo argonfand -f 100 # enforce max speed and exit
sudo argonfand # start service
```

ArgonOne case fan is quite noisy so for your own comfort it should never be exposed to direct light and settings should set speed to 0 if temperature is below 55.


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
# comments starts with "#" 
# TEMP=SPEED
49=0
50=40
60=100
```

My settings (not default)

```toml
54=0
55=55
65=80
80=100
```
