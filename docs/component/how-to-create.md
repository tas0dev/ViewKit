---
title: "How to create components?"
file: "resources/components/*"
---

# How to create components?

Components are basically created using HTML and CSS.
The location where they are created is not yet defined.

Components must not have the `body`, `html`, or `head` attributes specified.

Doing so will cause them to malfunction.

When specifying styles, write the `style` tag **at the beginning of the file** and define the styles there.

When using tags such as `div`, be sure to specify a unique class using `class`.

Then, CSS should only be applied to that class.

Currently, there are two special tags (used only in ViewKit) used when creating components:

- `<Children />`
- `<Content />`

We will explain each in detail.

### Children tag
The Children tag primarily inserts the specified child elements into the location where it is written.

This is a frequently used tag, essential when displaying buttons in a Dock or placing text in a card.

### Content tag
The Content tag inserts data other than components, such as strings and images.
You can specify the content that can be inserted as Content. For example, to specify a string, you would write:
```html
<Content type="String" />
```
To insert an image, simply change String to Image.