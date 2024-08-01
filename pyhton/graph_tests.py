import numpy as np

from pyhton.LexRank import create_markov_matrix, connected_nodes


def main():
    print("Hello World!")
    # create numpy array of dimension 4x4 random 1 or zero
    # create a random array
    arr = np.random.randint(2, size=(4, 4))
    print(arr)
    markov_matrix = create_markov_matrix(arr)
    print(markov_matrix)

    groups = connected_nodes(markov_matrix)
    print(f"group: {groups}")

    lables = np.zeros(4)
    tag = 0
    wh_res = np.where(lables == tag)
    print(f"wh_res: {wh_res}")
    as_arr = np.asarray(lables == tag).nonzero()
    print(f"as_arr: {as_arr}")


if __name__ == "__main__":
    main()
