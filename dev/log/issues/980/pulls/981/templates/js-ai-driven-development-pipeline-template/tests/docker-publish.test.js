import { describe, it, expect } from 'test-anywhere';
import { readFileSync } from 'node:fs';

import { evaluateDockerPublishConfig } from '../scripts/check-docker-publish.mjs';
import { parseArgs, waitForNpmVersion } from '../scripts/wait-for-npm.mjs';

const releaseWorkflow = readFileSync('.github/workflows/release.yml', 'utf8');
const dockerHubAction = readFileSync(
  '.github/actions/publish-dockerhub/action.yml',
  'utf8'
);

function getWorkflowJob(workflow, jobName) {
  const lines = workflow.replaceAll('\r\n', '\n').split('\n');
  const start = lines.findIndex((line) => line === `  ${jobName}:`);
  if (start === -1) {
    return '';
  }

  const nextJob = lines.findIndex(
    (line, index) => index > start && /^[ ]{2}[a-zA-Z0-9_-]+:\s*$/.test(line)
  );
  return lines.slice(start, nextJob === -1 ? lines.length : nextJob).join('\n');
}

function expectOrdered(text, markers) {
  let lastIndex = -1;

  for (const marker of markers) {
    const index = text.indexOf(marker);
    expect(index).toBeGreaterThan(lastIndex);
    lastIndex = index;
  }
}

describe('optional Docker Hub publishing workflow', () => {
  it('adds a Docker publish job downstream of npm release jobs', () => {
    const dockerJob = getWorkflowJob(releaseWorkflow, 'docker-publish');

    expect(dockerJob).toContain('needs: [release, instant-release]');
    expect(dockerJob).toContain('DOCKERHUB_IMAGE: ${{ vars.DOCKERHUB_IMAGE }}');
    expect(dockerJob).toContain(
      'DOCKERHUB_USERNAME: ${{ vars.DOCKERHUB_USERNAME }}'
    );
    expect(dockerJob).toContain(
      'DOCKERHUB_TOKEN: ${{ secrets.DOCKERHUB_TOKEN }}'
    );
    expect(dockerJob).toContain(
      'RELEASE_VERSION: ${{ needs.release.outputs.published_version || needs.instant-release.outputs.published_version }}'
    );
  });

  it('waits for the exact npm version before publishing Docker tags', () => {
    const dockerJob = getWorkflowJob(releaseWorkflow, 'docker-publish');

    expectOrdered(dockerJob, [
      '- name: Check Docker publish configuration',
      '- name: Wait for npm package availability before Docker publish',
      '- name: Publish Docker image to Docker Hub',
    ]);
    expect(dockerJob).toContain(
      'node scripts/wait-for-npm.mjs --release-version "${{ env.RELEASE_VERSION }}"'
    );
    expect(dockerHubAction).toContain('type=raw,value=${{ inputs.version }}');
    expect(dockerHubAction).toContain(
      'org.opencontainers.image.version=${{ inputs.version }}'
    );
    expect(dockerHubAction).toContain(
      'NPM_PACKAGE_VERSION=${{ inputs.version }}'
    );
  });

  it('uses Docker official GitHub Actions with DOCKERHUB_TOKEN authentication', () => {
    expect(dockerHubAction).toContain(
      'uses: ./.github/actions/setup-buildx-resilient'
    );
    expect(dockerHubAction).toContain('uses: docker/login-action@v4');
    expect(dockerHubAction).toContain('password: ${{ inputs.token }}');
    expect(dockerHubAction).toContain('uses: docker/metadata-action@v6');
    expect(dockerHubAction).toContain('uses: docker/build-push-action@v7');
  });
});

describe('Docker publish configuration', () => {
  it('keeps Docker publishing disabled until DOCKERHUB_IMAGE is configured', () => {
    const config = evaluateDockerPublishConfig({
      env: {},
    });

    expect(config.enabled).toBe(false);
    expect(config.errors).toEqual([]);
  });

  it('reports missing Docker Hub credentials when publishing is enabled', () => {
    const config = evaluateDockerPublishConfig({
      cwd: '.',
      env: {
        DOCKERFILE: 'package.json',
        DOCKERHUB_IMAGE: 'owner/image',
      },
    });

    expect(config.enabled).toBe(false);
    expect(config.errors).toContain(
      'DOCKERHUB_USERNAME is required when DOCKERHUB_IMAGE is set'
    );
    expect(config.errors).toContain(
      'DOCKERHUB_TOKEN is required when DOCKERHUB_IMAGE is set'
    );
  });

  it('accepts complete Docker Hub configuration without exposing the token', () => {
    const config = evaluateDockerPublishConfig({
      cwd: '.',
      env: {
        DOCKER_CONTEXT: '.',
        DOCKERFILE: 'package.json',
        DOCKERHUB_IMAGE: 'owner/image',
        DOCKERHUB_TOKEN: 'secret-token',
        DOCKERHUB_USERNAME: 'owner',
      },
    });

    expect(config).toEqual({
      context: '.',
      dockerfile: 'package.json',
      enabled: true,
      errors: [],
      image: 'owner/image',
      username: 'owner',
    });
  });
});

describe('wait-for-npm.mjs', () => {
  it('parses release wait options', () => {
    expect(
      parseArgs(
        [
          '--release-version',
          '1.2.3',
          '--package-name=@scope/pkg',
          '--max-attempts',
          '2',
          '--sleep-seconds=1',
        ],
        {}
      )
    ).toEqual({
      jsRoot: '',
      maxAttempts: 2,
      packageName: '@scope/pkg',
      releaseVersion: '1.2.3',
      sleepSeconds: 1,
    });
  });

  it('retries until npm reports the requested version', async () => {
    let attempts = 0;
    const sleeps = [];

    const available = await waitForNpmVersion({
      checkAvailability(packageName, version) {
        attempts++;
        expect(packageName).toBe('@scope/pkg');
        expect(version).toBe('1.2.3');
        return attempts === 2;
      },
      maxAttempts: 3,
      packageName: '@scope/pkg',
      sleepFn(seconds) {
        sleeps.push(seconds);
        return Promise.resolve();
      },
      sleepSeconds: 1,
      stdout() {},
      version: '1.2.3',
    });

    expect(available).toBe(true);
    expect(sleeps).toEqual([1]);
  });
});
