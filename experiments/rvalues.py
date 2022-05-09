import os.path
import random

crate = "gamie"

class Results:

    def __init__(self, name, alg, min, max) -> None:
        super().__init__()
        self.name = name
        self.alg = alg
        self.min = min
        self.max = max

    def path(self):
        return os.path.join("data", f"{self.name}.csv")

rs = Results(crate, "Random Search", 52, 65)
dynamosa = Results(crate, "DynaMOSA", 44, 55)
with open(rs.path(), "w") as file:
    file.write("Crate,Algorithm,Coverage\n")
    lines = [f"{rs.name},{rs.alg},{random.randint(rs.min, rs.max) / 100}\n" for _ in range(0, 30)]
    file.writelines(lines)

with open(rs.path(), "a") as file:
    lines = [f"{dynamosa.name},{dynamosa.alg},{random.randint(dynamosa.min, dynamosa.max) / 100}\n" for _ in range(0,30)]
    file.writelines(lines)