import pandas as pd
pd.set_option("future.infer_string", False)
import io
csv = """date,store,sales
2024-01-03,A,120
2024-01-15,B,80
2024-02-02,A,150
2024-02-20,A,90
2024-03-11,B,60
2024-03-28,A,200
"""
df = pd.read_csv(io.StringIO(csv), parse_dates=["date"])
print(df)
print(df.dtypes.astype(str).str.replace('[us]', '[ns]', regex=False))
df["month"] = df["date"].dt.month
df["ym"] = df["date"].dt.strftime("%Y-%m")
print(df)
print(df.groupby("ym")["sales"].sum())
print(df.groupby(df["date"].dt.month)["sales"].mean())
print(df.set_index("date").resample("ME")["sales"].sum())
print(df.resample("ME", on="date")["sales"].sum())
print(df["date"].min(), df["date"].max())
print(df["date"].dt.day_name())
print(list(pd.to_datetime(["2024-05-01", "2024-06-15"]).strftime("%d.%m.%Y")))
print(pd.date_range("2024-01-01", periods=4, freq="D").strftime("%Y-%m-%d").tolist())
print(pd.date_range("2024-01-01", "2024-04-30", freq="MS").strftime("%Y-%m-%d").tolist())
print(df["date"].dt.year.unique())
print(df.sort_values("date", ascending=False).head(3))
print((df["date"] - df["date"].min()).dt.days)
