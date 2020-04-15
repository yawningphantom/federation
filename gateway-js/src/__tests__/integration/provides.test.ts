import { execute, overrideResolversInService } from '../execution-utils';
import { fixtures } from 'apollo-federation-integration-testsuite';

it('does not have to go to another service when field is given', async () => {
  const query = `#graphql
    query GetReviewers {
      topReviews {
        author {
          username
        }
      }
    }
  `;

  const { data, queryPlan } = await execute( {
    query,
  });

  expect(data).toEqual({
    topReviews: [
      { author: { username: '@ada' } },
      { author: { username: '@ada' } },
      { author: { username: '@complete' } },
      { author: { username: '@complete' } },
      { author: { username: '@complete' } },
    ],
  });

  expect(queryPlan).not.toCallService('accounts');
  expect(queryPlan).toCallService('reviews');
});

it('does not have to go to another service when field is given for interface', async () => {
  const query = gql`
    query GetReviewers {
      topReviews(first: 3) {
        product {
          manufacturer
        }
      }
    }
  `;

  const { data,errors, queryPlan } = await execute(
    [accounts, books, inventory, product, reviews],
    {
      query,
    },
  );

  expect(data).toEqual({
    topReviews: [
      { product: { manufacturer: 'factoryA' } },
      { product: { manufacturer: 'factoryB' } },
      { product: { manufacturer: null } },
    ],
  });

  expect(errors).toBeUndefined()
  expect(queryPlan).not.toCallService('product');
  expect(queryPlan).not.toCallService('book');
  expect(queryPlan).toCallService('reviews');
});

it('does not load fields provided even when going to other service', async () => {
  const [accounts, ...restFixtures] = fixtures;

  const username = jest.fn();
  const localAccounts = overrideResolversInService(accounts, {
    User: {
      username,
    },
  });

  const query = `#graphql
    query GetReviewers {
      topReviews {
        author {
          username
          name {
            first
            last
          }
        }
      }
    }
  `;

  const { data, queryPlan } = await execute(
    {
      query,
    },
    [localAccounts, ...restFixtures],
  );

  expect(data).toEqual({
    topReviews: [
      { author: { username: '@ada', name: { first: 'Ada', last: 'Lovelace' } } },
      { author: { username: '@ada', name: { first: 'Ada', last: 'Lovelace' } } },
      { author: { username: '@complete', name: { first: 'Alan', last: 'Turing' } } },
      { author: { username: '@complete', name: { first: 'Alan', last: 'Turing' } } },
      { author: { username: '@complete', name: { first: 'Alan', last: 'Turing' } } },
    ],
  });

  expect(username).not.toHaveBeenCalled();
  expect(queryPlan).toCallService('accounts');
  expect(queryPlan).toCallService('reviews');
});
