import pandas as pd
pd.set_option("future.infer_string", False)
import io
csv = """region,product,qty,price,date
North,apple,3,1.25,2024-01-05
South,pear,5,0.8,2024-01-17
North,pear,2,0.85,2024-02-01
East,apple,,1.3,2024-02-11
South,apple,7,1.2,2024-03-03
North,plum,1,2.5,2024-03-21
"""
df = pd.read_csv(io.StringIO(csv))
print(df)
print(df.dtypes)
print(df.shape, len(df), list(df.columns))
print(df.head(2))
print(df["qty"])
print(df[df["price"] > 1])
print(df.describe())
print(df.groupby("region")["price"].sum())
print(df.groupby("region").agg({"price": "mean", "qty": "sum"}))
print(df.sort_values("price", ascending=False))
print(df["product"].value_counts())
print(df.isna().sum())
print(df.fillna(0))
print(df.dropna())
print(df.rename(columns={"qty": "quantity"}).columns)
print(df.astype({"qty": "float"}).dtypes)
