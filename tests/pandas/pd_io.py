import pandas as pd
pd.set_option("future.infer_string", False)
import json, os
os.makedirs("/tmp/pdtest", exist_ok=True)
with open("/tmp/pdtest/a.csv", "w") as f:
    f.write('id;name;value\n1;"Smith, J";3,5\n2;Doe;4,25\n3;"Q ""x""";\n')
df = pd.read_csv("/tmp/pdtest/a.csv", sep=";", decimal=",")
print(df)
print(df.dtypes)
df.to_csv("/tmp/pdtest/b.csv", index=False)
print(open("/tmp/pdtest/b.csv").read())
with open("/tmp/pdtest/r.json", "w") as f:
    json.dump([{"a": 1, "b": "x", "c": {"d": 2}}, {"a": 2, "b": "y", "c": {"d": 3}}], f)
j = pd.read_json("/tmp/pdtest/r.json")
print(j)
print(pd.json_normalize(json.load(open("/tmp/pdtest/r.json"))))
print(j.to_json())
print(j[["a", "b"]].to_json(orient="records"))
print(j[["a", "b"]].to_markdown())
print(pd.DataFrame({"x": [1.5, 2.0]}).to_markdown(index=False))
print(pd.read_csv("/tmp/pdtest/b.csv", usecols=["id", "value"], nrows=2))
print(pd.read_csv("/tmp/pdtest/b.csv", index_col="id"))
print(pd.read_csv("/tmp/pdtest/b.csv", dtype={"id": str}).dtypes)
