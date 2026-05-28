# 5 Go Tips for Beginners

A collection of practical tips for new Go developers.

## 1. Use `defer` for Cleanup

```go
f, err := os.Open("file.txt")
if err != nil {
    return err
}
defer f.Close()
```

Always close resources right after opening them — `defer` makes this natural.

## 2. Prefer Composition Over Inheritance

Go doesn't have classes. Use struct embedding instead:

```go
type Logger struct {
    prefix string
}

func (l *Logger) Log(msg string) {
    fmt.Printf("[%s] %s\n", l.prefix, msg)
}

type Service struct {
    Logger  // embedded
    name    string
}
```

## 3. Handle Errors Explicitly

Don't ignore errors. Every error value tells you something:

```go
data, err := os.ReadFile("config.json")
if err != nil {
    log.Fatalf("failed to read config: %v", err)
}
```

## 4. Use Interfaces Sparingly

Define interfaces where you consume them, not where you implement them. Small interfaces (1-3 methods) are best.

## 5. Leverage `go fmt`

Run `go fmt ./...` before every commit. Consistent formatting eliminates bike-shedding.

---

Happy coding!
