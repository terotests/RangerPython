# The kind of analysis script a model writes for a slide.
import pandas as pd
pd.set_option("future.infer_string", False)
import io, json

raw = """order_id,date,customer,country,product,category,qty,unit_price,discount
1001,2024-01-04,Acme,Finland,Widget,Hardware,10,19.99,0.0
1002,2024-01-09,Globex,Sweden,Gadget,Hardware,4,49.5,0.1
1003,2024-01-15,Initech,Finland,Service plan,Services,1,299.0,0.0
1004,2024-02-02,Acme,Finland,Gadget,Hardware,2,49.5,0.05
1005,2024-02-11,Umbrella,Norway,Widget,Hardware,25,18.5,0.15
1006,2024-02-19,Globex,Sweden,Service plan,Services,2,299.0,0.0
1007,2024-03-01,Hooli,Denmark,Widget,Hardware,8,19.99,0.0
1008,2024-03-07,Initech,Finland,Gizmo,Accessories,15,7.25,0.0
1009,2024-03-22,Umbrella,Norway,Gizmo,Accessories,40,6.9,0.2
1010,2024-03-30,Hooli,Denmark,Service plan,Services,1,299.0,0.1
"""
df = pd.read_csv(io.StringIO(raw), parse_dates=["date"])
df["revenue"] = df["qty"] * df["unit_price"] * (1 - df["discount"])
df["month"] = df["date"].dt.to_period("M").astype(str)

print("Rows:", len(df))
print(f"Total revenue: {df['revenue'].sum():,.2f}")
print(f"Average order: {df['revenue'].mean():.2f}")

by_country = (df.groupby("country", as_index=False)["revenue"].sum()
                .sort_values("revenue", ascending=False))
print(by_country)

monthly = df.groupby("month")["revenue"].sum().reset_index(name="total")
print(monthly)
monthly["growth_pct"] = monthly["total"].pct_change().mul(100).round(1)
print(monthly)

pivot = df.pivot_table(index="country", columns="category", values="revenue", aggfunc="sum", fill_value=0).round(0)
print(pivot)

top_customers = df.groupby("customer")["revenue"].sum().nlargest(3)
for name, value in top_customers.items():
    print(f"{name}: {value:.1f}")

share = (df.groupby("category")["revenue"].sum() / df["revenue"].sum() * 100).round(1)
print(share.sort_values(ascending=False).to_dict())

summary = df.groupby("category").agg(
    orders=("order_id", "count"),
    units=("qty", "sum"),
    revenue=("revenue", "sum"),
    avg_discount=("discount", "mean"),
).sort_values("revenue", ascending=False)
print(summary.round(2))

big = df[(df["revenue"] > 200) & (df["country"].isin(["Finland", "Sweden"]))]
print(big[["order_id", "customer", "revenue"]].to_string(index=False))

chart = {"labels": by_country["country"].tolist(), "values": [round(v, 2) for v in by_country["revenue"]]}
print(json.dumps(chart))
print(df.describe().round(2))
print(pd.crosstab(df["country"], df["category"]))
print(df.groupby(["country", "category"]).size().unstack(fill_value=0))
print(df["customer"].nunique(), df["product"].unique().tolist() if hasattr(df["product"].unique(), "tolist") else df["product"].unique())
rows = []
for _, r in df.head(3).iterrows():
    rows.append(f"{r['customer']} bought {r['qty']} x {r['product']}")
print("\n".join(rows))
print(df.loc[df["revenue"].idxmax(), "customer"])
df2 = df.copy()
df2.loc[df2["country"] == "Finland", "region"] = "Home"
df2["region"] = df2["region"].fillna("Export")
print(df2["region"].value_counts())
print(df.sort_values(["country", "revenue"], ascending=[True, False]).groupby("country").head(1)[["country", "customer", "revenue"]])
