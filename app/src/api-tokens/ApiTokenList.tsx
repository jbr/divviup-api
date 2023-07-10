import Breadcrumb from "react-bootstrap/Breadcrumb";
import Row from "react-bootstrap/Row";
import Col from "react-bootstrap/Col";
import Button from "react-bootstrap/Button";
import { AccountBreadcrumbs, WithAccount } from "../util";
import {
  Check,
  Clipboard,
  ClipboardCheckFill,
  PencilFill,
  ShieldLock,
  ShieldLockFill,
  Trash,
} from "react-bootstrap-icons";
import { Suspense, useCallback, useEffect, useState } from "react";
import {
  Await,
  Form,
  useActionData,
  useFetcher,
  useLoaderData,
  useNavigation,
} from "react-router-dom";
import { ApiToken } from "../ApiClient";
import Table from "react-bootstrap/Table";
import React from "react";
import { DateTime } from "luxon";
import FormControl from "react-bootstrap/FormControl";
import FormGroup from "react-bootstrap/FormGroup";
import InputGroup from "react-bootstrap/InputGroup";
import Modal from "react-bootstrap/Modal";
import Placeholder from "react-bootstrap/Placeholder";
import OverlayTrigger from "react-bootstrap/OverlayTrigger";
import Tooltip from "react-bootstrap/Tooltip";

export default function ApiTokens() {
  const navigation = useNavigation();
  return (
    <>
      <Breadcrumbs />
      <Row>
        <Col>
          <h1>
            <ShieldLock />{" "}
            <Suspense fallback={<Placeholder animation="glow" xs={6} />}>
              <WithAccount>{(account) => account.name}</WithAccount>
            </Suspense>{" "}
            API Tokens
          </h1>
        </Col>
      </Row>
      <Row className="mb-3">
        <Col>
          <Form method="post">
            <Button
              variant="primary"
              type="submit"
              disabled={navigation.state === "submitting"}
            >
              <ShieldLockFill /> Create Token
            </Button>
          </Form>
        </Col>
      </Row>
      <Row>
        <Col>
          <ApiTokenList />
        </Col>
      </Row>
    </>
  );
}

function Breadcrumbs() {
  return (
    <AccountBreadcrumbs>
      <Breadcrumb.Item active>ApiTokens</Breadcrumb.Item>
    </AccountBreadcrumbs>
  );
}

function ApiTokenList() {
  let { apiTokens } = useLoaderData() as {
    apiTokens: Promise<ApiToken[]>;
  };

  return (
    <Table>
      <thead>
        <tr>
          <td>Token Name</td>
          <td></td>
          <td>Last Used</td>
          <td>Created</td>
          <td></td>
        </tr>
      </thead>
      <tbody>
        <Suspense>
          <Await resolve={apiTokens}>
            {(apiTokens: ApiToken[]) =>
              apiTokens.map((apiToken) => (
                <ApiTokenRow key={apiToken.id} apiToken={apiToken} />
              ))
            }
          </Await>
        </Suspense>
      </tbody>
    </Table>
  );
}

function TokenName({ apiToken }: { apiToken: ApiToken }) {
  let [isEditing, setEditing] = useState(false);
  let edit = useCallback(() => setEditing(true), [setEditing]);
  let fetcher = useFetcher();
  useEffect(() => {
    if (fetcher.data) setEditing(false);
  }, [fetcher, setEditing]);
  if (isEditing) {
    return (
      <fetcher.Form action={apiToken.id} method="patch">
        <FormGroup>
          <InputGroup>
            <FormControl
              type="text"
              name="name"
              defaultValue={apiToken.name}
              data-1p-ignore
              autoFocus
            />
            <Button type="submit">
              <Check />
            </Button>
          </InputGroup>
        </FormGroup>
      </fetcher.Form>
    );
  } else {
    return (
      <span onClick={edit}>
        {apiToken.name || `Token ${apiToken.token_hash.slice(0, 5)}`}{" "}
        <Button
          variant="outline-secondary"
          onClick={edit}
          size="sm"
          className="ml-auto"
        >
          <PencilFill />
        </Button>
      </span>
    );
  }
}

function RelativeTime({ time, missing }: { time?: string; missing?: string }) {
  return time ? (
    <relative-time datetime={time} format="relative">
      {DateTime.fromISO(time).toLocal().toLocaleString(DateTime.DATETIME_SHORT)}
    </relative-time>
  ) : (
    <>{missing || "never"}</>
  );
}

function DeleteButton({ apiToken }: { apiToken: ApiToken }) {
  const navigation = useNavigation();

  const [show, setShow] = useState(false);
  const close = React.useCallback(() => setShow(false), []);
  const open = React.useCallback(() => setShow(true), []);
  const fetcher = useFetcher();

  useEffect(() => {
    if (fetcher.data) close();
  }, [fetcher, close]);

  return (
    <>
      <Button
        variant="outline-danger"
        className="ml-auto"
        size="sm"
        onClick={open}
      >
        <Trash />
      </Button>
      <Modal show={show} onHide={close}>
        <Modal.Header closeButton>
          <Modal.Title>Confirm Token Deletion {apiToken.name}</Modal.Title>
        </Modal.Header>
        <Modal.Body>
          This token will immediately be revoked.{" "}
          {apiToken.last_used_at ? (
            <>
              It was last used <RelativeTime time={apiToken.last_used_at} />
            </>
          ) : (
            <>It has never been used</>
          )}
        </Modal.Body>
        <Modal.Footer>
          <Button variant="secondary" onClick={close}>
            Close
          </Button>
          <fetcher.Form method="delete" action={apiToken.id}>
            <Button
              variant="danger"
              type="submit"
              disabled={navigation.state === "submitting"}
            >
              <Trash /> Delete
            </Button>
          </fetcher.Form>
        </Modal.Footer>
      </Modal>
    </>
  );
}

function Token({ token }: { token: string | null }) {
  if (!token) return null;
  const [copied, setCopied] = useState(false);
  const copy = useCallback(() => {
    navigator.clipboard.writeText(token).then(() => {
      setCopied(true);
    });
  }, [setCopied, token]);

  return (
    <OverlayTrigger
      overlay={<Tooltip>{copied ? "Copied!" : "Click to copy"}</Tooltip>}
    >
      <span onClick={copy} style={{ cursor: "pointer" }}>
        <code className="user-select-all">{token}</code>{" "}
        <Button size="sm" variant="outline-secondary" className="ml-auto">
          {copied ? <ClipboardCheckFill /> : <Clipboard />}
        </Button>
      </span>
    </OverlayTrigger>
  );
}

function ApiTokenRow({ apiToken }: { apiToken: ApiToken }) {
  const actionData = useActionData() as
    | undefined
    | (ApiToken & { token: string });
  const token = actionData?.id == apiToken.id ? actionData?.token : null;

  return (
    <tr className={token ? "table-success" : ""}>
      <td>
        <TokenName apiToken={apiToken} />
      </td>
      <td>
        <Token token={token} />
      </td>
      <td>
        <RelativeTime time={apiToken.last_used_at} />
      </td>
      <td>
        <RelativeTime time={apiToken.created_at} />
      </td>
      <td>
        <DeleteButton apiToken={apiToken} />
      </td>
    </tr>
  );
}