import fast_text_splitter as fts


# Test the splitter
# addd main function
def main():
    # Test the splitter
    conf_params = fts.py_ws_config_params(None,["\n\n", "\n"],8,2,True);
    data = "Hello, you all! \n How are you foo and poo? \n\n I am fine. Nice to meet you all insecure!"
    splits = fts.text_split_ws(data, conf_params)
    for split in splits:
        print("Split")
        print(split.split_strings)

    hf_conf_params = fts.py_hf_config_params(None,None,["\n\n", "\n"],32,2,True);

    splits = fts.text_split_hf(data, hf_conf_params)
    for split in splits:
        print("Split")
        print(split.split_strings)


if __name__ == "__main__":
    main()
