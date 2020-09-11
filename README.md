# climacell_exporter

Prometheus exporter for Climacell

# Usage

```
./climacell_exporter --token <climacell token> -lat <latitude> -lon <longitude>
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