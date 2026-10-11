import pandas as pd
pd.set_option("future.infer_string", False)
df = pd.DataFrame({
    "country": ["FI", "SE", "FI", "NO", "SE", "FI", "NO", "DK"],
    "product": ["a", "b", "b", "a", "a", "c", "b", "a"],
    "revenue": [100, 250, 80, 120, 60, 300, 90, 40],
    "units": [1, 3, 2, 2, 1, 5, 1, 1],
    "margin": [0.2, 0.35, 0.1, 0.25, 0.3, 0.4, 0.15, 0.05],
})
summary = df.groupby("country").agg({"revenue": "sum"}).sort_values("revenue", ascending=False)
print(summary)
top = df.groupby("product")["revenue"].sum().sort_values(ascending=False).head(10)
print(top)
print(df.groupby(["country", "product"])["revenue"].sum())
print(df.groupby("country").agg(total=("revenue", "sum"), avg_margin=("margin", "mean"), n=("units", "count")))
print(df.groupby("country")["revenue"].agg(["sum", "mean", "max"]))
print(df.groupby("country").size())
print(df.groupby("country", as_index=False)["revenue"].sum())
print(df.groupby("country")[["revenue", "units"]].mean())
print(df.groupby("product").revenue.mean().round(1))
print(df.groupby("country")["revenue"].transform("sum"))
print(df.groupby("country").count())
print(df.groupby("country")["product"].nunique())
print(df.groupby("country")["revenue"].max().idxmax())
for k, g in df.groupby("product"):
    print(k, len(g), g["revenue"].sum())
print(df.groupby("country")["revenue"].sum().reset_index())
print(df.groupby("country").agg({"revenue": ["sum", "mean"], "units": "max"}))
