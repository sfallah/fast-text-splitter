import fast_text_splitter as fts


# Test the splitter
# addd main function
def main():
    # Test the splitter
    conf_params = fts.py_ws_config_params(None,["\n\n", "\n"],8,2,False);
    splits = fts.text_split_ws("Hello, you all! \n How are you foo and poo? \n\n I am fine. Nice to meet you all insecure!", conf_params)
    for split in splits:
        print("Split")
        print(split.split_strings)


if __name__ == "__main__":
    main()
