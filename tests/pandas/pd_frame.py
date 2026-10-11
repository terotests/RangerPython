import pandas as pd
pd.set_option("future.infer_string", False)
df = pd.DataFrame([
    {"name": "alpha", "score": 81.5, "age": 31, "team": "red", "active": True},
    {"name": "beta", "score": 92.25, "age": 45, "team": "blue", "active": False},
    {"name": "gamma", "score": None, "age": 28, "team": "red", "active": True},
    {"name": "delta", "score": 67.0, "age": 39, "team": "green", "active": True},
    {"name": "eps", "score": 74.75, "age": 52, "team": "blue", "active": False},
])
print(df)
print(df.info())
print(df.columns)
print(df.index)
print(df["name"].tolist())
df["score2"] = df["score"] * 2
df["senior"] = df["age"] >= 40
print(df)
print(df.loc[1])
print(df.loc[df["team"] == "red", ["name", "score"]])
print(df.iloc[1:3])
print(df.iloc[0, 1])
print(df.loc[2, "age"])
df.loc[df["score"].isna(), "score"] = 0
print(df)
print(df.set_index("name"))
print(df.set_index("name").loc["beta"])
print(df.drop(columns=["score2"]))
print(df.drop_duplicates(subset=["team"]))
print(df.select_dtypes(include="number"))
print(df.sort_values(["team", "age"], ascending=[True, False]))
print(df.nlargest(2, "score"))
print(df.query("age > 30 and team == 'blue'"))
print(df.assign(ratio=lambda d: d["score"] / d["age"]).round(2))
print(df.apply(lambda r: r["name"].upper() + "-" + r["team"], axis=1))
print(df[["age", "score"]].sum())
print(df[["age", "score"]].mean())
print(df.to_dict("records")[:2])
print(df.to_csv(index=False))
print(df.head(2).to_json(orient="records"))
print(df.T)
print(df.melt(id_vars=["name"], value_vars=["age", "score"]).head(4))
print(df["team"].value_counts(normalize=True))
print(df.describe(include="all"))
print(df.corr(numeric_only=True))
