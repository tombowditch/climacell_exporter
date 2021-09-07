# climacell_exporter

Prometheus exporter for Tomorrow.io (previously known as Climacell)
(The name of the exporter has not been updated to allow for backwards compatibility)

# Docker

https://github.com/users/tombowditch/packages/container/package/climacell_exporter

# Usage

```
./climacell_exporter --token <tomorrow.io api v4 token> -lat <latitude> -lon <longitude>
```

# Environment variables

You can use environment variables instead of command line arguments

```
TOKEN=xxxxxyyyyyyzzzzz
LAT=123
LON=321
```

# Alertmanager

climacell_exporter keeps the `climacell_state` metric up to date with scrape status so you can alert on scrape fails:

```
        - alert: ClimacellFailed
          expr: climacell_state < 1
          for: 5m
          annotations:
            summary: "Climacell API not returning data"
            description: "Investigate error log for climacell_exporter"
```