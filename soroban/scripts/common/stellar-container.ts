/**
 * Copyright 2026 Circle Internet Group, Inc. All rights reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

import { ResultAsync } from "neverthrow";
import { GenericContainer, StartedTestContainer, Wait } from "testcontainers";
import { faker } from "@faker-js/faker";

// ==================== Types ====================

export interface QuickstartUrls {
  horizonUrl: URL;
  rpcUrl: URL;
  friendbotUrl: URL;
  host: string;
  port: number;
}

export interface StartedQuickstartContainer {
  container: StartedTestContainer;
  urls: QuickstartUrls;
  stop: () => Promise<void>;
}

export interface QuickstartConfig {
  image?: string;
  name?: string;
  reuseExisting?: boolean;
  /** Stream container log lines to stdout during startup. Defaults to false. */
  debug?: boolean;
}

// ==================== Constants ====================

const DEFAULT_IMAGE = "stellar/quickstart:latest";
const QUICKSTART_PORT = 8000;

// ==================== Container Management ====================

/**
 * Starts a Stellar Quickstart container for e2e testing.
 *
 * Uses testcontainers for automatic lifecycle management:
 * - Container starts with proper health checks
 * - Auto-removed on stop
 * - Dynamic port allocation
 *
 * @example
 * ```typescript
 * const result = await startQuickstartContainer({ limits: "unlimited" });
 * if (result.isErr()) throw result.error;
 *
 * const { urls, stop } = result.value;
 * const server = new rpc.Server(urls.rpcUrl.href);
 *
 * // ... run tests ...
 *
 * await stop();
 * ```
 */
export const startQuickstartContainer = (
  config: QuickstartConfig = {}
): ResultAsync<StartedQuickstartContainer, Error> => {
  const { image = DEFAULT_IMAGE, name, reuseExisting = false, debug = false } = config;

  const containerName = ["stellar-e2e", name ?? faker.word.adjective()].join("-");
  // Use --local for standalone ephemeral network with friendbot
  const args = ["--local"];

  return ResultAsync.fromThrowable(
    async () => {
      console.log(`Starting Stellar container: ${containerName}`);
      console.log(`Using image: ${image}`);
      console.log(`Args: ${args.join(" ")}`);

      let containerConfig = await new GenericContainer(image)
        .withName(containerName)
        .withCommand(args)
        .withExposedPorts(QUICKSTART_PORT)
        .withLogConsumer((stream) => {
          if (!debug) return;
          stream.on("data", (msg: string) => {
            const trimmed = msg.toString().trim();
            if (
              trimmed.includes("error") ||
              trimmed.includes("Error") ||
              trimmed.includes("Synced") ||
              trimmed.includes("ready") ||
              trimmed.includes("Starting") ||
              trimmed.includes("listening") ||
              trimmed.includes("horizon")
            ) {
              console.log(`[stellar] ${trimmed}`);
            }
          });
          stream.on("err", (fault: string) => {
            console.error(`[stellar-err] ${fault.toString().trim()}`);
          });
        })
        .withWaitStrategy(
          Wait.forAll([
            // RPC HTTP check (GET returns 4xx but proves it's listening)
            Wait.forHttp("/rpc", QUICKSTART_PORT).forStatusCodeMatching((code) => code < 500),
            // Friendbot HTTP check (returns 400 without params, but proves it's listening)
            Wait.forHttp("/friendbot", QUICKSTART_PORT).forStatusCodeMatching((code) => code < 500),
          ])
        )
        .withStartupTimeout(300_000)
        .withAutoRemove(true);

        if (reuseExisting) {
          containerConfig = containerConfig.withReuse();
        }
        const container = await containerConfig.start();

      const host = container.getHost();
      const port = container.getMappedPort(QUICKSTART_PORT);
      const baseUrl = `http://${host}:${port}`;

      const urls: QuickstartUrls = {
        horizonUrl: new URL(baseUrl),
        rpcUrl: new URL(`${baseUrl}/soroban/rpc`),
        friendbotUrl: new URL(`${baseUrl}/friendbot`),
        host,
        port,
      };

      console.log(`Stellar container ready at ${baseUrl}`);

      return {
        container,
        urls,
        stop: async () => {
          console.log(`Stopping Stellar container: ${containerName}`);
          await container.stop();
        },
      };
    },
    (e) => new Error(`Failed to start Stellar container: ${(e as Error).message}`, { cause: e })
  )();
};
