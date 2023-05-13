#!/usr/bin/env python
import json


with open("../conversions.json") as f:
    CONVERSIONS = json.load(f)


def shakespeareanize(text):
    words = text.split()
    shakespearean_text = []

    for word in words:
        if word.isupper():
            shakespearean_text.append(CONVERSIONS.get(word.lower(), word))

        elif word[-1] in [",", ".", "!", "?", ";", ":"]:
            punctuation = word[-1]
            word = word[:-1]
            shakespearean_text.append(CONVERSIONS.get(word.lower(), word) + punctuation)

        else:
            shakespearean_text.append(CONVERSIONS.get(word.lower(), word))

    return " ".join(shakespearean_text)


def main():
    text = input("What do you want to shakespeareanize?\n> ")
    result = shakespeareanize(text)
    print(result)


if __name__ == "__main__":
    main()
