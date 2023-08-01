# Pon specification

Pon is a minimalistic programming language that is intended for very high-level programs.

## Language goals

* Be readable: everyone should be able to read Pon code, even if they know nothing about the language
* Be simple: everyone should be able to write Pon code after a short introduction, even if they had no prior knowledge of Pon. Pon should not intimidate the user with new terms and complicated concepts. Instead, Pon should use things that the user is likely to know already

## Terms

### Action

Names of actions are case-insensitive

```pon
display {Hello, world!}
```

#### Regular action

```pon
action {print {the text} {the amount} times} {
    repeat {the amount} times {
        print {the text}
    }
}
```

They can also return things through

let {helo} be {hello}

#### Special action


