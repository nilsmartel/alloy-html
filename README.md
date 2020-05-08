# Alloy

Programmer friendly syntax for html files.

---

```css
html.h-100 {
    head {
        meta(charset: UTF-8)
        link(
            rel: stylesheet # new line as a valid separator
            href: https://stackpath.bootstrapcdn.com/bootstrap/4.3.1/css/bootstrap.min.css
        )

        style {
            .h-100 {
                height: 100%
            }
        }
    }

    body {
        div#header.w-100(style: "height: 48px; margin-top: 8px") {
                                                    #     ________ <- Note how the opening and clsoing parens are still getting counted
            img(src: ../ressources/icon.png, onclick: goto('home'))

            h2.color-green { Graphmasters }
        }
    }
}
```
