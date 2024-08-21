import torch
from sentence_transformers import SentenceTransformer


def is_mps_available_info():
    if not torch.backends.mps.is_available():
        if not torch.backends.mps.is_built():
            print("MPS not available because the current PyTorch install was not "
                  "built with MPS enabled.")
        else:
            print("MPS not available because the current MacOS version is not 12.3+ "
                  "and/or you do not have an MPS-enabled device on this machine.")
    else:
        print("MPS is available on this machine.")


def main():
    is_mps_available_info()
    # model = SentenceTransformer("intfloat/multilingual-e5-large-instruct", device="mps")
    model = SentenceTransformer("sentence-transformers/all-MiniLM-L6-v2")
    # Two lists of sentences
    sentences1 = [
        "The new movie is awesome",
        "The cat sits outside",
        "A man is playing guitar",
        "I love pasta",
    ]

    sentences2 = [
        "The dog plays in the garden",
        "The new movie is so great",
        "A woman watches TV",
        "Do you like pizza?",
    ]

    # Compute embeddings for both lists
    embeddings1 = model.encode(sentences1, device="mps")
    embeddings2 = model.encode(sentences2, device="mps")

    # Compute cosine similarities
    similarities = model.similarity(embeddings1, embeddings2)

    # Output the pairs with their score
    for idx_i, sentence1 in enumerate(sentences1):
        print(sentence1)
        for idx_j, sentence2 in enumerate(sentences2):
            print(f" - {sentence2: <30}: {similarities[idx_i][idx_j]:.4f}")


if __name__ == "__main__":
    main()
