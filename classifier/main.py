import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns

from sklearn.ensemble import StackingClassifier, RandomForestClassifier
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import accuracy_score, precision_score, recall_score, f1_score, roc_auc_score, confusion_matrix
from sklearn.model_selection import train_test_split
from sklearn.naive_bayes import GaussianNB
from sklearn.preprocessing import LabelEncoder
from sklearn.tree import DecisionTreeClassifier


def load_dataset():
    X = pd.read_csv('dataset/Obfuscated-MalMem2022.csv')
    y = X['Class']
    X.pop('Class')
    X.pop('Category')

    X_train, X_test, y_train, y_test = train_test_split(X, y, train_size=0.8, shuffle=True,
                                                                random_state=37)
    return X_train, X_test, y_train, y_test


def load_validation_set():
    X = pd.read_csv('dataset/Validation.csv')
    y = X['Class']
    X.pop('Class')
    X.pop('Category')

    return X, y


def show_metrics(y_true: list[int], y_pred: list[int]):
    print(f'Accuracy: {accuracy_score(y_true, y_pred)}')
    print(f'Precision: {precision_score(y_true, y_pred)}')
    print(f'Recall: {recall_score(y_true, y_pred)}')
    print(f'F1-score: {f1_score(y_true, y_pred)}')
    print(f'Area under ROC curve: {roc_auc_score(y_true, y_pred)}')


def show_confusion_matrix(y_true: pd.Series, y_pred: np.ndarray, title: str, label_encoder: LabelEncoder) -> None:
    y_true = label_encoder.inverse_transform(y_true)
    y_pred = label_encoder.inverse_transform(y_pred)
    labels = label_encoder.classes_
    cm = confusion_matrix(y_true, y_pred, labels=labels)

    fig = plt.figure(figsize=(8, 6))
    sns.heatmap(cm, annot=True, fmt='d', cmap='Purples', xticklabels=labels,
                yticklabels=labels)
    plt.xlabel('Predicted')
    plt.ylabel('Actual')
    plt.title(title)
    plt.show()



le = LabelEncoder()
X_train, X_test, y_train, y_test = load_dataset()
y_train = le.fit_transform(y_train)
y_test = le.transform(y_test)


classifier = StackingClassifier(estimators=[('nb', GaussianNB()), ('rf', RandomForestClassifier()), ('dt', DecisionTreeClassifier())], final_estimator=LogisticRegression())
classifier.fit(X_train, y_train)

y_pred = classifier.predict(X_test)
print('Test set')
show_metrics(y_test, y_pred)
show_confusion_matrix(y_test, y_pred, 'Test set', le)

X_vali, y_vali = load_validation_set()
y_vali = le.transform(y_vali)
y_vali_pred = classifier.predict(X_vali)

print('Validation set')

show_metrics(y_vali, y_vali_pred)
show_confusion_matrix(y_vali, y_vali_pred, 'Validation set', le)
