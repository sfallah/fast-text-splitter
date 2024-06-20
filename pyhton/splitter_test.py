import fast_text_splitter as fts


# Test the splitter
# addd main function
def main():
    # Test the splitter
    patterns = [[".", "!", "?"]]
    ws_splitter = fts.create_ws_splitter(patterns=patterns, max_tokens=10, merge_level=0, parallel=False)
    text = "Hello, world! How are you doing today? I am doing fine."
    print(ws_splitter.splits(text))

    hf_splitter = fts.create_hf_splitter(patterns=patterns, max_tokens=12, merge_level=0, parallel=False)
    text = "Hello, world! How are you doing today? I am doing fine."
    for res in hf_splitter.splits(text):
        print(res.text)
        print(res.tokens)

    # Test the splitter
    new_text = "Once upon a time there was a rabbit. The rabbit was very fast. The rabbit was very happy."
    print(len(new_text.encode('utf-8')))
    for res in hf_splitter.splits(new_text):
        print(res.text)
        print(res.tokens)


if __name__ == "__main__":
    main()
