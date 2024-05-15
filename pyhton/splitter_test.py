import fast_text_splitter as fts


# Test the splitter
# addd main function
def main():
    # Test the splitter
    splits = fts.text_split_ws("Hello, you all! \n How are you foo and poo? \n\n I am fine. Nice to meet you all insecure!")
    for split in splits:
        print(split.split_strings)


if __name__ == "__main__":
    main()
