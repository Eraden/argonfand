# ArgonOne Fan Service

## Requirements


* Create:
  `/etc/modules-load.d/raspi-conf.conf`

  and fill it with `i2c-dev`

* Add to `/boot/config.txt`

  must contain

  ```
  dtparam=i2c_arm=on
  dtparam=i2s=on
  ```

  Example:

  ```
  enable_gic=1
  dtparam=i2c_arm=on
  dtoverlay=vc4-kms-v3d,i2c-rtc,ds1307,pcf85063
  initramfs initramfs-linux.img followkernel
  enable_uart=1
  ```

Raspberry PI must be fully restarted after this so i2c module will be loaded.

## Run

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

`/etc/argonfand.toml`

Format:

```toml
verbose = false
delay = 1000

[[values]]
temp = 45
speed = 0

[[values]]
temp = 54
speed = 10

[[values]]
temp = 55
speed = 50

[[values]]
temp = 65
speed = 80

[[values]]
temp = 80
speed = 100
```

