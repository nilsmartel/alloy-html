# Alloy

Programmer friendly syntax for html files.

---

## Example

```c
// vim: set syntax=c :
// DOCTYPE html
html.h-100 {
    head {
        meta(charset: UTF-8);
        link(
            rel: stylesheet,
            href: "https://stackpath.bootstrapcdn.com/bootstrap/4.3.1/css/bootstrap.min.css"
        );

        style "
            .h-100 {
                height: 100%
            }
        "
    }

    body {
        p(x:'noob') 'hello world'
        p(a: true, b: false);
        p.pretty(a: true, b: false) "everything is nice"

        div#header.w-100(style: "height: 48px; margin-top: 8px") {
                                                    //   ________ <- Note how the opening and closing parens are still getting counted
            img(src: ../ressources/icon.png, onclick: goto('home'));

            h2.color-green { "Graphmasters" }
            input(type: "text");
        }
    }
}
```

## Why not react?

Because I think writing raw html is an underrated and legitimate way of coding highly performant website.
React has many downsides and I'd avoid using webpack any day of the week for small projects.
