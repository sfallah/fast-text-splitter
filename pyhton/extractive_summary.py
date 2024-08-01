import fast_text_splitter as fts
import numpy as np
from LexRank import degree_centrality_scores
from sentence_transformers import SentenceTransformer, SimilarityFunction


def main():
    model = SentenceTransformer("all-MiniLM-L6-v2")
    #model = SentenceTransformer("BAAI/bge-large-en-v1.5", similarity_fn_name=SimilarityFunction.DOT_PRODUCT)
    #model = SentenceTransformer("dunzhang/stella_en_400M_v5")
    #model = SentenceTransformer("BAAI/bge-m3")

    # read the text from the file tests/test_data/superlinear.txt
    with open("../tests/test_data/superlinear.txt", "r") as f:
        text = f.read()
        patterns = [["\n\n"], ["\n"], [".", "!", "?"]]
        fts_splitter = fts.create_hf_splitter(patterns=patterns, max_tokens=512, merge_level=None, parallel=True, model=None)
        fts_sub_splitter = fts.create_hf_splitter(patterns=patterns, max_tokens=512, merge_level=3, parallel=True, model=None)
        #fts_splitter = fts.create_hf_splitter(patterns=patterns, max_tokens=512, merge_level=None, parallel=True, model="BAAI/bge-large-en-v1.5")
        #fts_sub_splitter = fts.create_hf_splitter(patterns=patterns, max_tokens=512, merge_level=3, parallel=True, model="BAAI/bge-large-en-v1.5")
        split_num = 0
        for split in fts_splitter.splits(text):
            print(f"#### Split {split_num} ####")
            print(f"no-tokens: {len(split.tokens)}")

            split_num += 1
            sentences = []
            for sub_res in fts_sub_splitter.splits(split.text):
                #print(f"no-tokens: {len(sub_res.tokens)}")
                #print(sub_res.text)
                sentences.append(sub_res.text)

            #embeddings = model.encode(sentences, prompt="Retrieve semantically similar text: ")
            embeddings = model.encode(sentences)
            similarity_scores = model.similarity(embeddings, embeddings).numpy()

            # Compute the centrality for each sentence
            #centrality_scores = degree_centrality_scores(similarity_scores, threshold=0.7)
            centrality_scores = degree_centrality_scores(similarity_scores, threshold=None)
            most_central_sentence_indices = np.argsort(-centrality_scores)
            print("scores :", centrality_scores.tolist())
            print("most_central_sentence_indices :", most_central_sentence_indices.tolist())

            # We argsort so that the first element is the sentence with the highest score

            # Print the 5 sentences with the highest scores
            print("\n\nSummary:")
            for idx in most_central_sentence_indices[0:2]:
                print(sentences[idx].strip())
            print("\n\nSplit Text:")
            print(split.text)


if __name__ == "__main__":
    main()
